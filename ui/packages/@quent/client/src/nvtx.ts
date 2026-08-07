// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { keepPreviousData, queryOptions, useQuery } from '@tanstack/react-query';
import type { NvtxViewportRequest } from '@quent/utils';
import { fetchEngineContexts, fetchNvtxCatalog, fetchNvtxViewport } from './api';
import { DEFAULT_STALE_TIME } from './constants';
import { canonicalizeNvtxRequest } from './nvtxCanonical';
export { canonicalizeNvtxRequest, canonicalizeNvtxSelections } from './nvtxCanonical';

export const engineContextsQueryOptions = (engineId: string) =>
  queryOptions({
    queryKey: ['engineContexts', engineId],
    queryFn: () => fetchEngineContexts(engineId),
    staleTime: DEFAULT_STALE_TIME,
  });

export const nvtxCatalogQueryOptions = (contextId: string) =>
  queryOptions({
    queryKey: ['nvtxCatalog', contextId],
    queryFn: () => fetchNvtxCatalog(contextId),
    staleTime: Infinity,
  });

export const nvtxViewportQueryOptions = (
  contextId: string,
  request: NvtxViewportRequest,
  options?: { enabled?: boolean; staleTime?: number }
) => {
  const canonical = canonicalizeNvtxRequest(request);
  const selectionKey = canonical.selections.map(selection => [
    selection.domain_id.toString(10),
    selection.category_ids,
    selection.include_uncategorized,
  ]);
  return queryOptions({
    queryKey: [
      'nvtxViewport',
      contextId,
      canonical.viewport.start.toString(10),
      canonical.viewport.end.toString(10),
      selectionKey,
    ],
    queryFn: () => fetchNvtxViewport(contextId, canonical),
    enabled: options?.enabled ?? true,
    staleTime: options?.staleTime ?? DEFAULT_STALE_TIME,
    placeholderData: keepPreviousData,
  });
};

export const useEngineContexts = (engineId: string) =>
  useQuery(engineContextsQueryOptions(engineId));

export const useNvtxCatalog = (contextId: string) => useQuery(nvtxCatalogQueryOptions(contextId));

export const useNvtxViewport = (
  contextId: string,
  request: NvtxViewportRequest,
  options?: { enabled?: boolean; staleTime?: number }
) => useQuery(nvtxViewportQueryOptions(contextId, request, options));
