// 코어 커맨드 셋의 호출과 타입 — API 문서(PAW-API-001)의 전사. invoke를 부르는 유일한 파일이다
import { Channel, invoke } from '@tauri-apps/api/core'

export type Period = 'all' | '1y' | '6m' | '3m'
export type SortKey = 'weight' | 'registered'
export type RegionFilter = { kind: 'all' } | { kind: 'sido'; name: string } | { kind: 'unknown' }
export type LinkKind = 'pawinhand' | 'source'

export interface Condition {
  minWeightKg: number
  period: Period
  region: RegionFilter
  sort: SortKey
}

export interface LoadResult {
  /** 다 받은 시각, ISO 8601에 시간대까지 */
  fetchedAt: string
  /** 공고번호 중복을 뺀 마릿수 */
  total: number
}

export type RegionCount = { kind: 'sido'; name: string; count: number } | { kind: 'unknown'; count: number }

export interface AnimalView {
  noticeNo: string
  registeredOn: string
  weight: { kg: number; raw: string }
  notice: { start: string; end: string; status: 'notice' | 'protected' }
  region: { sido: string | null; sigungu: string | null }
  photos: string[]
  hasSourceNotice: boolean
  breed: string | null
  color: string | null
  age: string | null
  sex: 'M' | 'F' | 'Q' | null
  neutered: 'Y' | 'N' | 'U' | null
  feature: string | null
  foundAt: string | null
  shelter: { name: string | null; address: string | null; tel: string | null }
  office: { name: string | null; tel: string | null }
}

export interface QueryResult {
  count: number
  allCount: number
  regionCounts: RegionCount[]
  regionReset: boolean
  animals: AnimalView[]
}

export type ErrorCode =
  | 'connection-failed'
  | 'timeout'
  | 'bad-format'
  | 'too-many-pages'
  | 'no-snapshot'
  | 'invalid-condition'
  | 'not-found'
  | 'no-source'
  | 'not-allowed'
  | 'open-failed'

export interface ApiError {
  code: ErrorCode
  message: string
  url?: string
}

export function sameRegion(a: RegionFilter, b: RegionFilter): boolean {
  if (a.kind === 'sido' && b.kind === 'sido') return a.name === b.name
  return a.kind === b.kind
}

export function sameCondition(a: Condition, b: Condition): boolean {
  return a.minWeightKg === b.minWeightKg && a.period === b.period && a.sort === b.sort && sameRegion(a.region, b.region)
}

/** 코어가 아닌 곳(인자 모양 오류 등)에서 온 거절도 ApiError 모양으로 맞춘다 */
export function toApiError(error: unknown): ApiError {
  if (typeof error === 'object' && error !== null && 'code' in error) return error as ApiError
  return { code: 'bad-format', message: String(error) }
}

async function call<T>(command: string, args: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw toApiError(error)
  }
}

/** 포인핸드에서 보호중 고양이를 전부 받는다. 쪽마다 지금까지 받은 마릿수로 onProgress를 부른다 */
export function loadAnimals(onProgress: (received: number) => void): Promise<LoadResult> {
  const channel = new Channel<{ received: number }>()
  channel.onmessage = (message) => onProgress(message.received)
  return call('load_animals', { onProgress: channel })
}

export function queryAnimals(condition: Condition): Promise<QueryResult> {
  return call('query_animals', { condition })
}

export function openLink(noticeNo: string, kind: LinkKind): Promise<{ url: string }> {
  return call('open_link', { noticeNo, kind })
}
