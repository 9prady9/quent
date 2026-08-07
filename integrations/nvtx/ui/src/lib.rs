// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! UI-facing NVTX contracts and model-to-viewport conversion.
//!
//! The exchange types deliberately contain presentation semantics, not capture
//! internals: domain/category selection, lane identities, nesting depth,
//! clipped display bounds, and viewport-scoped statistics are all resolved here.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fmt;

use nvtx_analyzer::{NvtxColor, NvtxModel, NvtxSpan, SpanId, SpanKind};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Stable metadata for one NVTX stream.
#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxCatalog {
    pub trace_start: u64,
    pub trace_end: u64,
    pub domains: Vec<NvtxCatalogDomain>,
}

/// Selectable metadata for one domain.
#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxCatalogDomain {
    pub domain_id: u64,
    pub name: String,
    pub color: String,
    pub threads: Vec<NvtxCatalogThread>,
    pub categories: Vec<NvtxCatalogCategory>,
    pub has_uncategorized: bool,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxCatalogThread {
    pub thread_id: u32,
    pub name: String,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxCatalogCategory {
    pub category_id: u32,
    pub name: String,
}

/// Inclusive viewport bounds in Unix nanoseconds.
#[derive(TS, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NvtxViewportWindow {
    pub start: u64,
    pub end: u64,
}

/// One domain's selected categories.
#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NvtxDomainSelection {
    pub domain_id: u64,
    pub category_ids: Vec<u32>,
    pub include_uncategorized: bool,
}

/// Request for one atomically-scoped set of lanes and statistics.
#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NvtxViewportRequest {
    pub viewport: NvtxViewportWindow,
    pub selections: Vec<NvtxDomainSelection>,
}

/// UI-ready NVTX content for one viewport and selection.
#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxViewportResponse {
    pub viewport: NvtxViewportWindow,
    pub domains: Vec<NvtxDomainLaneGroup>,
    pub statistics: Vec<NvtxRangeStatistics>,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxDomainLaneGroup {
    pub domain_id: u64,
    pub name: String,
    pub color: String,
    pub lanes: Vec<NvtxLane>,
}

/// A truthful NVTX lane identity. Thread depth rows are explicit rather than
/// reconstructed by TypeScript.
#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NvtxLaneIdentity {
    Thread { thread_id: u32, depth: u32 },
    Process,
    Marks,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxLane {
    pub id: String,
    pub label: String,
    pub identity: NvtxLaneIdentity,
    pub ranges: Vec<NvtxRangeItem>,
    pub marks: Vec<NvtxMarkItem>,
}

#[derive(TS, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NvtxRangeKind {
    PushPop,
    StartEnd,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxRangeItem {
    pub message: String,
    pub domain_id: u64,
    pub domain_name: String,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub color: String,
    pub kind: NvtxRangeKind,
    pub thread_id: Option<u32>,
    pub thread_name: Option<String>,
    /// The actual captured start, retained for tooltip truthfulness.
    pub observed_start: u64,
    /// The actual captured close; absent for an incomplete range.
    pub observed_end: Option<u64>,
    /// Bounds clipped to the requested viewport for rendering.
    pub display_start: u64,
    pub display_end: u64,
    pub observed_duration: Option<u64>,
    pub incomplete: bool,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxMarkItem {
    pub message: String,
    pub domain_id: u64,
    pub domain_name: String,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub color: String,
    pub timestamp: u64,
}

/// Statistics over exactly the filtered, intersecting range population.
/// Closed durations are clipped to the visible window; incomplete ranges are
/// counted but never assigned an inferred duration.
#[derive(TS, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvtxRangeStatistics {
    pub message: String,
    pub domain_id: u64,
    pub domain_name: String,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub count: u64,
    pub observed_count: u64,
    pub total_duration: u64,
    pub avg_duration: u64,
    pub min_duration: u64,
    pub max_duration: u64,
    pub saturated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvtxViewportError {
    InvalidWindow,
    EmptySelection { domain_id: u64 },
    DuplicateDomain { domain_id: u64 },
    UnknownDomain { domain_id: u64 },
    UnknownCategory { domain_id: u64, category_id: u32 },
    UncategorizedUnavailable { domain_id: u64 },
}

impl fmt::Display for NvtxViewportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWindow => write!(f, "viewport start must not exceed viewport end"),
            Self::EmptySelection { domain_id } => {
                write!(f, "domain {domain_id} selects no categories")
            }
            Self::DuplicateDomain { domain_id } => {
                write!(f, "domain {domain_id} appears more than once")
            }
            Self::UnknownDomain { domain_id } => write!(f, "unknown domain {domain_id}"),
            Self::UnknownCategory {
                domain_id,
                category_id,
            } => write!(f, "unknown category {category_id} in domain {domain_id}"),
            Self::UncategorizedUnavailable { domain_id } => {
                write!(f, "domain {domain_id} has no uncategorized items")
            }
        }
    }
}

impl Error for NvtxViewportError {}

impl NvtxCatalog {
    pub fn from_model(model: &NvtxModel) -> Self {
        let thread_names: HashMap<u32, &str> = model
            .threads()
            .iter()
            .map(|thread| (thread.thread_id, thread.name.as_str()))
            .collect();

        let mut domains = model
            .domains()
            .iter()
            .map(|domain| {
                let mut thread_ids = BTreeSet::new();
                let mut has_uncategorized = false;
                for span in model
                    .spans()
                    .iter()
                    .filter(|span| span.domain == domain.domain)
                {
                    if let SpanKind::PushPop { thread_id, .. } = span.kind {
                        thread_ids.insert(thread_id);
                    }
                    if is_range(span) && span.category.is_none() {
                        has_uncategorized = true;
                    }
                }
                has_uncategorized |= model
                    .marks()
                    .iter()
                    .any(|mark| mark.domain == domain.domain && mark.category.is_none());

                let mut threads: Vec<_> = thread_ids
                    .into_iter()
                    .map(|thread_id| NvtxCatalogThread {
                        thread_id,
                        name: thread_names.get(&thread_id).map_or_else(
                            || model.thread_name(thread_id),
                            |name| (*name).to_owned(),
                        ),
                    })
                    .collect();
                threads.sort_by(|left, right| {
                    left.name
                        .cmp(&right.name)
                        .then(left.thread_id.cmp(&right.thread_id))
                });

                let mut categories: Vec<_> = model
                    .categories()
                    .iter()
                    .filter(|category| category.domain == domain.domain)
                    .map(|category| NvtxCatalogCategory {
                        category_id: category.category,
                        name: category.name.clone(),
                    })
                    .collect();
                categories.sort_by(|left, right| {
                    left.name
                        .cmp(&right.name)
                        .then(left.category_id.cmp(&right.category_id))
                });

                NvtxCatalogDomain {
                    domain_id: domain.domain,
                    name: domain.name.clone(),
                    color: fallback_color(domain.domain).to_owned(),
                    threads,
                    categories,
                    has_uncategorized,
                }
            })
            .collect::<Vec<_>>();

        domains.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then(left.domain_id.cmp(&right.domain_id))
        });

        Self {
            trace_start: model.trace_start(),
            trace_end: model.trace_end(),
            domains,
        }
    }

    /// The explicit initial UI state: every catalog option selected.
    pub fn select_all(&self) -> Vec<NvtxDomainSelection> {
        let mut selections: Vec<_> = self
            .domains
            .iter()
            .filter_map(|domain| {
                let category_ids = domain
                    .categories
                    .iter()
                    .map(|category| category.category_id)
                    .collect::<Vec<_>>();
                (!category_ids.is_empty() || domain.has_uncategorized).then_some(
                    NvtxDomainSelection {
                        domain_id: domain.domain_id,
                        category_ids,
                        include_uncategorized: domain.has_uncategorized,
                    },
                )
            })
            .collect();
        selections.sort_by_key(|selection| selection.domain_id);
        selections
    }

    /// Validate and canonicalize selections for request bodies and cache keys.
    pub fn canonicalize_request(
        &self,
        mut request: NvtxViewportRequest,
    ) -> Result<NvtxViewportRequest, NvtxViewportError> {
        if request.viewport.start > request.viewport.end {
            return Err(NvtxViewportError::InvalidWindow);
        }

        let catalog: HashMap<_, _> = self
            .domains
            .iter()
            .map(|domain| (domain.domain_id, domain))
            .collect();
        let mut seen = BTreeSet::new();
        for selection in &mut request.selections {
            if !seen.insert(selection.domain_id) {
                return Err(NvtxViewportError::DuplicateDomain {
                    domain_id: selection.domain_id,
                });
            }
            if selection.category_ids.is_empty() && !selection.include_uncategorized {
                return Err(NvtxViewportError::EmptySelection {
                    domain_id: selection.domain_id,
                });
            }
            let domain =
                catalog
                    .get(&selection.domain_id)
                    .ok_or(NvtxViewportError::UnknownDomain {
                        domain_id: selection.domain_id,
                    })?;
            selection.category_ids.sort_unstable();
            selection.category_ids.dedup();
            for category_id in &selection.category_ids {
                if !domain
                    .categories
                    .iter()
                    .any(|category| category.category_id == *category_id)
                {
                    return Err(NvtxViewportError::UnknownCategory {
                        domain_id: selection.domain_id,
                        category_id: *category_id,
                    });
                }
            }
            if selection.include_uncategorized && !domain.has_uncategorized {
                return Err(NvtxViewportError::UncategorizedUnavailable {
                    domain_id: selection.domain_id,
                });
            }
        }
        request
            .selections
            .sort_by_key(|selection| selection.domain_id);
        Ok(request)
    }
}

impl NvtxViewportResponse {
    pub fn from_model(
        model: &NvtxModel,
        request: NvtxViewportRequest,
    ) -> Result<Self, NvtxViewportError> {
        let catalog = NvtxCatalog::from_model(model);
        let request = catalog.canonicalize_request(request)?;
        let selections: HashMap<_, _> = request
            .selections
            .iter()
            .map(|selection| (selection.domain_id, selection))
            .collect();
        let depths = span_depths(model);
        let mut statistics = BTreeMap::<StatsGroupKey, StatisticsAccumulator>::new();
        let mut domains = Vec::new();

        for domain in &catalog.domains {
            let Some(selection) = selections.get(&domain.domain_id) else {
                continue;
            };

            let mut thread_lanes = BTreeMap::<(u32, u32), Vec<NvtxRangeItem>>::new();
            let mut process_ranges = Vec::new();
            for (index, span) in model.spans().iter().enumerate() {
                if span.domain != domain.domain_id
                    || !is_range(span)
                    || !selected(selection, span.category)
                    || !intersects(
                        span.start,
                        span.end.unwrap_or(model.trace_end()),
                        request.viewport,
                    )
                {
                    continue;
                }

                let item = range_item(model, domain, span, request.viewport);
                statistics
                    .entry(StatsGroupKey {
                        domain_id: span.domain,
                        category_id: span.category,
                        message: span.name.clone(),
                    })
                    .or_default()
                    .accumulate(span, request.viewport);
                match span.kind {
                    SpanKind::PushPop { thread_id, .. } => {
                        thread_lanes
                            .entry((thread_id, depths[index]))
                            .or_default()
                            .push(item);
                    }
                    SpanKind::StartEnd => process_ranges.push(item),
                    SpanKind::Resource { .. } => unreachable!("resources were filtered above"),
                }
            }

            let mut marks = model
                .marks()
                .iter()
                .filter(|mark| {
                    mark.domain == domain.domain_id
                        && selected(selection, mark.category)
                        && mark.timestamp >= request.viewport.start
                        && mark.timestamp <= request.viewport.end
                })
                .map(|mark| NvtxMarkItem {
                    message: mark.name.clone(),
                    domain_id: domain.domain_id,
                    domain_name: domain.name.clone(),
                    category_id: mark.category,
                    category_name: mark
                        .category
                        .and_then(|id| model.category_name(domain.domain_id, id)),
                    color: display_color(mark.color, domain.domain_id),
                    timestamp: mark.timestamp,
                })
                .collect::<Vec<_>>();
            marks.sort_by(|left, right| {
                left.timestamp
                    .cmp(&right.timestamp)
                    .then(left.message.cmp(&right.message))
            });

            let thread_order: HashMap<_, _> = domain
                .threads
                .iter()
                .enumerate()
                .map(|(index, thread)| (thread.thread_id, index))
                .collect();
            let mut lane_entries: Vec<_> = thread_lanes.into_iter().collect();
            lane_entries.sort_by(
                |((left_thread, left_depth), _), ((right_thread, right_depth), _)| {
                    thread_order
                        .get(left_thread)
                        .cmp(&thread_order.get(right_thread))
                        .then(left_depth.cmp(right_depth))
                },
            );

            let mut lanes = lane_entries
                .into_iter()
                .map(|((thread_id, depth), mut ranges)| {
                    sort_ranges(&mut ranges);
                    let thread_name = model.thread_name(thread_id);
                    NvtxLane {
                        id: format!("nvtx:{}:thread:{thread_id}:depth:{depth}", domain.domain_id),
                        label: if depth == 0 {
                            thread_name
                        } else {
                            format!("{thread_name} · depth {depth}")
                        },
                        identity: NvtxLaneIdentity::Thread { thread_id, depth },
                        ranges,
                        marks: Vec::new(),
                    }
                })
                .collect::<Vec<_>>();

            if !process_ranges.is_empty() {
                sort_ranges(&mut process_ranges);
                lanes.push(NvtxLane {
                    id: format!("nvtx:{}:process", domain.domain_id),
                    label: "Process ranges".to_owned(),
                    identity: NvtxLaneIdentity::Process,
                    ranges: process_ranges,
                    marks: Vec::new(),
                });
            }
            if !marks.is_empty() {
                lanes.push(NvtxLane {
                    id: format!("nvtx:{}:marks", domain.domain_id),
                    label: "Marks".to_owned(),
                    identity: NvtxLaneIdentity::Marks,
                    ranges: Vec::new(),
                    marks,
                });
            }
            if !lanes.is_empty() {
                domains.push(NvtxDomainLaneGroup {
                    domain_id: domain.domain_id,
                    name: domain.name.clone(),
                    color: domain.color.clone(),
                    lanes,
                });
            }
        }

        let domain_order: HashMap<_, _> = catalog
            .domains
            .iter()
            .enumerate()
            .map(|(index, domain)| (domain.domain_id, index))
            .collect();
        let mut statistics = statistics
            .into_iter()
            .map(|(key, accumulator)| {
                let domain = catalog
                    .domains
                    .iter()
                    .find(|domain| domain.domain_id == key.domain_id)
                    .expect("statistics only include catalog domains");
                accumulator.finish(&key, domain, model)
            })
            .collect::<Vec<_>>();
        statistics.sort_by(|left, right| {
            domain_order
                .get(&left.domain_id)
                .cmp(&domain_order.get(&right.domain_id))
                .then(left.category_name.cmp(&right.category_name))
                .then(left.category_id.cmp(&right.category_id))
                .then(left.message.cmp(&right.message))
        });

        Ok(Self {
            viewport: request.viewport,
            domains,
            statistics,
        })
    }
}

fn is_range(span: &NvtxSpan) -> bool {
    matches!(span.kind, SpanKind::PushPop { .. } | SpanKind::StartEnd)
}

fn selected(selection: &NvtxDomainSelection, category: Option<u32>) -> bool {
    match category {
        Some(id) => selection.category_ids.binary_search(&id).is_ok(),
        None => selection.include_uncategorized,
    }
}

fn intersects(start: u64, effective_end: u64, viewport: NvtxViewportWindow) -> bool {
    start <= viewport.end && effective_end >= viewport.start
}

fn span_depths(model: &NvtxModel) -> Vec<u32> {
    let mut depths = vec![0; model.spans().len()];
    for (index, slot) in depths.iter_mut().enumerate() {
        let mut depth = 0_u32;
        let mut parent = model.spans()[index].kind.parent();
        let mut remaining = model.spans().len();
        while let Some(SpanId(parent_index)) = parent {
            if remaining == 0 {
                break;
            }
            depth = depth.saturating_add(1);
            parent = model
                .spans()
                .get(parent_index)
                .and_then(|span| span.kind.parent());
            remaining -= 1;
        }
        *slot = depth;
    }
    depths
}

fn range_item(
    model: &NvtxModel,
    domain: &NvtxCatalogDomain,
    span: &NvtxSpan,
    viewport: NvtxViewportWindow,
) -> NvtxRangeItem {
    let effective_end = span.end.unwrap_or(model.trace_end());
    let thread_id = span.kind.thread_id();
    NvtxRangeItem {
        message: span.name.clone(),
        domain_id: span.domain,
        domain_name: domain.name.clone(),
        category_id: span.category,
        category_name: span
            .category
            .and_then(|id| model.category_name(span.domain, id)),
        color: display_color(span.color, span.domain),
        kind: match span.kind {
            SpanKind::PushPop { .. } => NvtxRangeKind::PushPop,
            SpanKind::StartEnd => NvtxRangeKind::StartEnd,
            SpanKind::Resource { .. } => unreachable!("resources do not render as ranges"),
        },
        thread_id,
        thread_name: thread_id.map(|id| model.thread_name(id)),
        observed_start: span.start,
        observed_end: span.end,
        display_start: span.start.max(viewport.start),
        display_end: effective_end.min(viewport.end),
        observed_duration: span.duration(),
        incomplete: span.end.is_none(),
    }
}

fn sort_ranges(ranges: &mut [NvtxRangeItem]) {
    ranges.sort_by(|left, right| {
        left.display_start
            .cmp(&right.display_start)
            .then(left.display_end.cmp(&right.display_end))
            .then(left.message.cmp(&right.message))
    });
}

fn display_color(color: Option<NvtxColor>, domain_id: u64) -> String {
    match color {
        Some(NvtxColor {
            color_type: 1,
            value,
        }) => {
            let alpha = value >> 24;
            let red = (value >> 16) & 0xff;
            let green = (value >> 8) & 0xff;
            let blue = value & 0xff;
            format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
        }
        _ => fallback_color(domain_id).to_owned(),
    }
}

fn fallback_color(domain_id: u64) -> &'static str {
    const COLORS: [&str; 12] = [
        "#2563eb", "#7c3aed", "#db2777", "#dc2626", "#ea580c", "#ca8a04", "#16a34a", "#0d9488",
        "#0891b2", "#4f46e5", "#9333ea", "#475569",
    ];
    COLORS[(domain_id % COLORS.len() as u64) as usize]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StatsGroupKey {
    domain_id: u64,
    category_id: Option<u32>,
    message: String,
}

#[derive(Debug, Default)]
struct StatisticsAccumulator {
    count: u64,
    observed_count: u64,
    total_duration: u64,
    min_duration: u64,
    max_duration: u64,
    saturated: bool,
}

impl StatisticsAccumulator {
    fn accumulate(&mut self, span: &NvtxSpan, viewport: NvtxViewportWindow) {
        self.count = self.count.saturating_add(1);
        let Some(end) = span.end else {
            return;
        };
        let duration = end
            .min(viewport.end)
            .saturating_sub(span.start.max(viewport.start));
        if self.observed_count == 0 {
            self.min_duration = duration;
            self.max_duration = duration;
        } else {
            self.min_duration = self.min_duration.min(duration);
            self.max_duration = self.max_duration.max(duration);
        }
        self.observed_count = self.observed_count.saturating_add(1);
        match self.total_duration.checked_add(duration) {
            Some(total) => self.total_duration = total,
            None => {
                self.total_duration = u64::MAX;
                self.saturated = true;
            }
        }
    }

    fn finish(
        self,
        key: &StatsGroupKey,
        domain: &NvtxCatalogDomain,
        model: &NvtxModel,
    ) -> NvtxRangeStatistics {
        NvtxRangeStatistics {
            message: key.message.clone(),
            domain_id: key.domain_id,
            domain_name: domain.name.clone(),
            category_id: key.category_id,
            category_name: key
                .category_id
                .and_then(|id| model.category_name(key.domain_id, id)),
            count: self.count,
            observed_count: self.observed_count,
            total_duration: self.total_duration,
            avg_duration: self
                .total_duration
                .checked_div(self.observed_count)
                .unwrap_or(0),
            min_duration: self.min_duration,
            max_duration: self.max_duration,
            saturated: self.saturated,
        }
    }
}

#[cfg(test)]
mod tests {
    use nvtx_analyzer::NvtxModelBuilder;
    use nvtx_bridge::NvtxEventEntity;
    use nvtx_events::{NvtxColor, NvtxEvent, NvtxEventAttributes, NvtxMessage};
    use quent_events::Event;
    use uuid::Uuid;

    use super::*;

    fn event(timestamp: u64, event: NvtxEvent) -> Event<NvtxEventEntity> {
        Event::new(Uuid::nil(), timestamp, NvtxEventEntity(event))
    }

    fn attributes(name: &str, category: u32, color: Option<NvtxColor>) -> NvtxEventAttributes {
        NvtxEventAttributes {
            category,
            color,
            message: Some(NvtxMessage::String(name.to_owned())),
            payload: None,
        }
    }

    fn model() -> NvtxModel {
        NvtxModelBuilder::build(vec![
            event(
                100,
                NvtxEvent::RangePush {
                    domain: 2,
                    thread_id: 7,
                    attributes: attributes("outer", 3, None),
                },
            ),
            event(
                120,
                NvtxEvent::RangePush {
                    domain: 2,
                    thread_id: 7,
                    attributes: attributes(
                        "inner",
                        3,
                        Some(NvtxColor {
                            color_type: 1,
                            value: 0x8040_2010,
                        }),
                    ),
                },
            ),
            event(
                180,
                NvtxEvent::RangePop {
                    domain: 2,
                    thread_id: 7,
                },
            ),
            event(
                200,
                NvtxEvent::RangePop {
                    domain: 2,
                    thread_id: 7,
                },
            ),
            event(
                210,
                NvtxEvent::RangeStart {
                    domain: 2,
                    range_id: 9,
                    attributes: attributes("open", 0, None),
                },
            ),
            event(
                250,
                NvtxEvent::Mark {
                    domain: 2,
                    attributes: attributes("boundary", 0, None),
                },
            ),
            event(
                250,
                NvtxEvent::RangeStart {
                    domain: 2,
                    range_id: 10,
                    attributes: attributes("instant", 0, None),
                },
            ),
            event(
                250,
                NvtxEvent::RangeEnd {
                    domain: 2,
                    range_id: 10,
                },
            ),
        ])
    }

    #[test]
    fn canonical_selection_rules_are_enforced() {
        let catalog = NvtxCatalog::from_model(&model());
        let canonical = catalog
            .canonicalize_request(NvtxViewportRequest {
                viewport: NvtxViewportWindow {
                    start: 100,
                    end: 250,
                },
                selections: vec![NvtxDomainSelection {
                    domain_id: 2,
                    category_ids: vec![3, 3],
                    include_uncategorized: true,
                }],
            })
            .expect("valid selection");
        assert_eq!(canonical.selections[0].category_ids, vec![3]);

        let duplicate = catalog.canonicalize_request(NvtxViewportRequest {
            viewport: canonical.viewport,
            selections: vec![
                canonical.selections[0].clone(),
                canonical.selections[0].clone(),
            ],
        });
        assert!(matches!(
            duplicate,
            Err(NvtxViewportError::DuplicateDomain { .. })
        ));

        let empty = catalog.canonicalize_request(NvtxViewportRequest {
            viewport: canonical.viewport,
            selections: vec![NvtxDomainSelection {
                domain_id: 2,
                category_ids: vec![],
                include_uncategorized: false,
            }],
        });
        assert!(matches!(
            empty,
            Err(NvtxViewportError::EmptySelection { .. })
        ));

        let none = catalog
            .canonicalize_request(NvtxViewportRequest {
                viewport: canonical.viewport,
                selections: vec![],
            })
            .expect("selecting nothing is valid");
        assert!(none.selections.is_empty());
    }

    #[test]
    fn viewport_preserves_truth_and_clips_display_and_statistics() {
        let model = model();
        let response = NvtxViewportResponse::from_model(
            &model,
            NvtxViewportRequest {
                viewport: NvtxViewportWindow {
                    start: 110,
                    end: 250,
                },
                selections: NvtxCatalog::from_model(&model).select_all(),
            },
        )
        .expect("valid viewport");

        let ranges = response.domains[0]
            .lanes
            .iter()
            .flat_map(|lane| &lane.ranges)
            .collect::<Vec<_>>();
        let outer = ranges
            .iter()
            .find(|range| range.message == "outer")
            .unwrap();
        assert_eq!(outer.observed_start, 100);
        assert_eq!(outer.observed_end, Some(200));
        assert_eq!(outer.display_start, 110);
        assert_eq!(outer.display_end, 200);

        let open = ranges.iter().find(|range| range.message == "open").unwrap();
        assert!(open.incomplete);
        assert_eq!(open.observed_end, None);
        assert_eq!(open.display_end, 250);
        assert_eq!(open.observed_duration, None);

        let outer_stats = response
            .statistics
            .iter()
            .find(|stats| stats.message == "outer")
            .unwrap();
        assert_eq!(
            outer_stats.total_duration, 90,
            "duration is viewport-clipped"
        );
        let open_stats = response
            .statistics
            .iter()
            .find(|stats| stats.message == "open")
            .unwrap();
        assert_eq!(open_stats.count, 1);
        assert_eq!(open_stats.observed_count, 0);
        assert_eq!(open_stats.total_duration, 0);

        let instant = ranges
            .iter()
            .find(|range| range.message == "instant")
            .unwrap();
        assert_eq!(instant.display_start, 250);
        assert_eq!(instant.display_end, 250);
        let instant_stats = response
            .statistics
            .iter()
            .find(|stats| stats.message == "instant")
            .unwrap();
        assert_eq!(instant_stats.observed_count, 1);
        assert_eq!(instant_stats.total_duration, 0);
    }

    #[test]
    fn depth_boundary_marks_and_argb_color_are_ui_ready() {
        let model = model();
        let response = NvtxViewportResponse::from_model(
            &model,
            NvtxViewportRequest {
                viewport: NvtxViewportWindow {
                    start: 120,
                    end: 250,
                },
                selections: NvtxCatalog::from_model(&model).select_all(),
            },
        )
        .expect("valid viewport");
        let lanes = &response.domains[0].lanes;
        assert!(lanes.iter().any(|lane| {
            lane.identity
                == NvtxLaneIdentity::Thread {
                    thread_id: 7,
                    depth: 1,
                }
                && lane.ranges[0].color == "#40201080"
        }));
        assert!(lanes.iter().any(|lane| {
            lane.identity == NvtxLaneIdentity::Marks && lane.marks[0].timestamp == 250
        }));
    }

    #[test]
    fn generated_contract_uses_bigint_for_u64() {
        let config = ts_rs::Config::default();
        let declaration = NvtxViewportRequest::decl(&config);
        assert!(declaration.contains("viewport: NvtxViewportWindow"));
        assert!(NvtxDomainSelection::decl(&config).contains("domain_id: bigint"));
        assert!(NvtxRangeItem::decl(&config).contains("observed_end: bigint | null"));
    }
}
