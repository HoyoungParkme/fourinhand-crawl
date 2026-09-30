import type { AnimalView } from '../api/animals'

/** 보호중은 초록 채움, 공고중은 초록 테두리 */
export default function StatusBadge({ status }: { status: AnimalView['notice']['status'] }) {
  return status === 'protected' ? <span className="st keep">보호중</span> : <span className="st notice">공고중</span>
}
