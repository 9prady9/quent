// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

import { afterEach, describe, expect, it, vi } from 'vitest';
import type { NvtxViewportRequest } from '@quent/utils';
import { fetchNvtxCatalog, fetchNvtxViewport } from './api';
import { canonicalizeNvtxRequest, nvtxCatalogQueryOptions, nvtxViewportQueryOptions } from './nvtx';

function stubFetch(response: Response) {
  const fetchMock = vi.fn().mockResolvedValue(response);
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

const request: NvtxViewportRequest = {
  viewport: { start: 9007199254740993n, end: 9007199254741993n },
  selections: [
    { domain_id: 9n, category_ids: [7, 3, 7], include_uncategorized: false },
    { domain_id: 2n, category_ids: [], include_uncategorized: true },
  ],
};

describe('NVTX client', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('treats catalog 404 as optional absence and fetches catalogs once', async () => {
    stubFetch(new Response(null, { status: 404, statusText: 'Not Found' }));
    await expect(fetchNvtxCatalog('context-1')).resolves.toBeNull();
    expect(nvtxCatalogQueryOptions('context-1').staleTime).toBe(Infinity);
  });

  it('propagates non-404 catalog failures', async () => {
    stubFetch(new Response(null, { status: 500, statusText: 'Internal Server Error' }));
    await expect(fetchNvtxCatalog('context-1')).rejects.toThrow(
      'API Error: 500 Internal Server Error'
    );
  });

  it('uses the same canonical selector order for body and query key', async () => {
    const fetchMock = stubFetch(
      new Response(
        '{"viewport":{"start":9007199254740993,"end":9007199254741993},"domains":[],"statistics":[]}',
        { status: 200 }
      )
    );
    const canonical = canonicalizeNvtxRequest(request);
    expect(canonical.selections.map(selection => selection.domain_id)).toEqual([2n, 9n]);
    expect(canonical.selections[1].category_ids).toEqual([3, 7]);

    await fetchNvtxViewport('context-1', request);
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(init.body).toBe(
      '{"viewport":{"start":9007199254740993,"end":9007199254741993},"selections":[{"domain_id":2,"category_ids":[],"include_uncategorized":true},{"domain_id":9,"category_ids":[3,7],"include_uncategorized":false}]}'
    );

    const options = nvtxViewportQueryOptions('context-1', request);
    expect(options.queryKey).toEqual([
      'nvtxViewport',
      'context-1',
      '9007199254740993',
      '9007199254741993',
      [
        ['2', [], true],
        ['9', [3, 7], false],
      ],
    ]);
    expect(options.placeholderData).toBeTypeOf('function');
  });

  it('normalizes even safe u64 response fields to bigint', async () => {
    stubFetch(
      new Response(
        '{"trace_start":1,"trace_end":2,"domains":[{"domain_id":3,"name":"d","color":"#000000","threads":[],"categories":[],"has_uncategorized":false}]}',
        { status: 200 }
      )
    );
    const catalog = await fetchNvtxCatalog('context-1');
    expect(catalog?.trace_start).toBe(1n);
    expect(catalog?.domains[0].domain_id).toBe(3n);
  });
});
