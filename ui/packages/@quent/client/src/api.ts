// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { parseJsonWithBigInt, stringifyJsonWithBigInt } from '@quent/utils';
import { getApiBaseUrl } from './config';
import { canonicalizeNvtxRequest } from './nvtxCanonical';
import type {
  QueryBundle,
  QueryGroup,
  Query,
  BulkTimelinesResponse,
  SingleTimelineRequest,
  SingleTimelineResponse,
  BulkTimelineRequest,
  CategoricalTimelineRequest,
  DataFlowTimelineBinned,
  QueryFilter,
  OperatorFilter,
  EntityRef,
  Engine,
  TimelineConfig,
  EntityListRequest,
  EntityListResponse,
  EngineContexts,
  NvtxCatalog,
  NvtxViewportRequest,
  NvtxViewportResponse,
} from '@quent/utils';

interface ApiFetchOptions {
  params?: Record<string, string | number | boolean>;
  fetchOptions?: RequestInit;
}

/**
 * Issues the request and returns the raw {@link Response} — internal helper
 * for fetchers that need to inspect the status code themselves.
 * @param endpoint - API endpoint to call
 * @param options - Optional params and fetch options
 */
async function apiFetchResponse(endpoint: string, options?: ApiFetchOptions): Promise<Response> {
  const { params, fetchOptions } = options ?? {};
  const searchParams = params
    ? `?${new URLSearchParams(Object.entries(params).map(([k, v]) => [k, String(v)]))}`
    : '';
  const url = `${getApiBaseUrl()}${endpoint}${searchParams}`;

  const defaultOptions: RequestInit = {
    headers: {},
  };

  // Only set Content-Type for requests with a body
  if (fetchOptions?.body) {
    defaultOptions.headers = {
      'Content-Type': 'application/json',
    };
  }

  return fetch(url, { ...defaultOptions, ...fetchOptions });
}

/**
 * Generic API fetch helper — internal, not exported from package barrel
 * @param endpoint - API endpoint to call
 * @param options - Optional params and fetch options
 */
async function apiFetch<T>(endpoint: string, options?: ApiFetchOptions): Promise<T> {
  const response = await apiFetchResponse(endpoint, options);

  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }

  const text = await response.text();
  return parseJsonWithBigInt<T>(text);
}

/**
 * Fetch query bundle from API endpoint
 * @param engineId - The engine ID
 * @param queryId - The query ID to fetch the bundle for
 */
export async function fetchQueryBundle(
  engineId: string,
  queryId: string
): Promise<QueryBundle<EntityRef>> {
  return apiFetch<QueryBundle<EntityRef>>(`/engines/${engineId}/query/${queryId}`);
}

export async function fetchListEngines(): Promise<Engine[]> {
  return apiFetch<Engine[]>('/engines', { params: { with_metadata: true } });
}

export async function fetchEngineContexts(engineId: string): Promise<EngineContexts> {
  return apiFetch<EngineContexts>(`/engines/${engineId}/contexts`);
}

/** Fetch stable NVTX metadata, resolving a 404 to optional absence. */
export async function fetchNvtxCatalog(contextId: string): Promise<NvtxCatalog | null> {
  const response = await apiFetchResponse(`/nvtx/contexts/${contextId}/catalog`);
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }
  return normalizeNvtxCatalog(parseJsonWithBigInt<NvtxCatalog>(await response.text()));
}

export async function fetchNvtxViewport(
  contextId: string,
  request: NvtxViewportRequest
): Promise<NvtxViewportResponse> {
  const canonical = canonicalizeNvtxRequest(request);
  const response = await apiFetchResponse(`/nvtx/contexts/${contextId}/viewport`, {
    fetchOptions: {
      method: 'POST',
      body: stringifyJsonWithBigInt(canonical),
    },
  });
  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }
  return normalizeNvtxViewport(parseJsonWithBigInt<NvtxViewportResponse>(await response.text()));
}

function asBigInt(value: bigint | number): bigint {
  return typeof value === 'bigint' ? value : BigInt(value);
}

function normalizeNvtxCatalog(catalog: NvtxCatalog): NvtxCatalog {
  return {
    ...catalog,
    trace_start: asBigInt(catalog.trace_start),
    trace_end: asBigInt(catalog.trace_end),
    domains: catalog.domains.map(domain => ({
      ...domain,
      domain_id: asBigInt(domain.domain_id),
    })),
  };
}

function normalizeNvtxViewport(viewport: NvtxViewportResponse): NvtxViewportResponse {
  return {
    ...viewport,
    viewport: {
      start: asBigInt(viewport.viewport.start),
      end: asBigInt(viewport.viewport.end),
    },
    domains: viewport.domains.map(domain => ({
      ...domain,
      domain_id: asBigInt(domain.domain_id),
      lanes: domain.lanes.map(lane => ({
        ...lane,
        ranges: lane.ranges.map(range => ({
          ...range,
          domain_id: asBigInt(range.domain_id),
          observed_start: asBigInt(range.observed_start),
          observed_end: range.observed_end === null ? null : asBigInt(range.observed_end),
          display_start: asBigInt(range.display_start),
          display_end: asBigInt(range.display_end),
          observed_duration:
            range.observed_duration === null ? null : asBigInt(range.observed_duration),
        })),
        marks: lane.marks.map(mark => ({
          ...mark,
          domain_id: asBigInt(mark.domain_id),
          timestamp: asBigInt(mark.timestamp),
        })),
      })),
    })),
    statistics: viewport.statistics.map(statistics => ({
      ...statistics,
      domain_id: asBigInt(statistics.domain_id),
      count: asBigInt(statistics.count),
      observed_count: asBigInt(statistics.observed_count),
      total_duration: asBigInt(statistics.total_duration),
      avg_duration: asBigInt(statistics.avg_duration),
      min_duration: asBigInt(statistics.min_duration),
      max_duration: asBigInt(statistics.max_duration),
    })),
  };
}

export async function fetchListCoordinators(engineId: string): Promise<QueryGroup[]> {
  return apiFetch<QueryGroup[]>(`/engines/${engineId}/query-groups`);
}

export async function fetchListQueries(engineId: string, coordinatorId: string): Promise<Query[]> {
  return apiFetch<Query[]>(`/engines/${engineId}/query_group/${coordinatorId}/queries`);
}

export async function fetchSingleTimeline(
  engineId: string,
  request: SingleTimelineRequest<QueryFilter, OperatorFilter>,
  durationSeconds: number
): Promise<SingleTimelineResponse> {
  return apiFetch<SingleTimelineResponse>(`/engines/${engineId}/timeline/single`, {
    params: { duration: durationSeconds },
    fetchOptions: {
      method: 'POST',
      body: JSON.stringify(request),
    },
  });
}

export async function fetchBulkTimelines(
  engineId: string,
  request: BulkTimelineRequest<QueryFilter, OperatorFilter>
): Promise<BulkTimelinesResponse> {
  return apiFetch<BulkTimelinesResponse>(`/engines/${engineId}/timeline/bulk`, {
    fetchOptions: {
      method: 'POST',
      body: JSON.stringify(request),
    },
  });
}

/**
 * Fetch a ranked, paged list of a query's entities (longest resource-usage
 * span first). Backs the long-entities Gantt view.
 */
export async function fetchEntityList(
  engineId: string,
  request: EntityListRequest<QueryFilter, OperatorFilter>
): Promise<EntityListResponse> {
  return apiFetch<EntityListResponse>(`/engines/${engineId}/entities`, {
    fetchOptions: {
      method: 'POST',
      body: JSON.stringify(request),
    },
  });
}

/**
 * Fetch the data-flow categorical timeline for a query (all operators in one
 * response). Resolves to `null` when the engine's analyzer does not implement
 * the data-flow protocol (HTTP 501) — an expected "feature unavailable"
 * outcome, not an error, so react-query settles instead of retrying.
 * @param measures - Measure names to compute; empty means all declared measures.
 */
export async function fetchDataFlow(
  engineId: string,
  queryId: string,
  config: TimelineConfig,
  measures: string[] = []
): Promise<DataFlowTimelineBinned | null> {
  const request: CategoricalTimelineRequest<QueryFilter> = {
    measures,
    config,
    app_params: { query_id: queryId },
  };
  const response = await apiFetchResponse(`/engines/${engineId}/timeline/data-flow`, {
    fetchOptions: {
      method: 'POST',
      body: JSON.stringify(request),
    },
  });
  if (response.status === 501) return null;
  if (!response.ok) {
    throw new Error(`API Error: ${response.status} ${response.statusText}`);
  }
  return parseJsonWithBigInt<DataFlowTimelineBinned>(await response.text());
}
