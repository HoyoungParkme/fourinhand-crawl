import { formatKg } from '../text/format'

/** 노란 무게표. 크기는 놓인 자리(카드·상세)가 정한다 */
export default function WeightTag({ kg }: { kg: number }) {
  return (
    <span className="kg">
      {formatKg(kg)}
      <small>kg</small>
    </span>
  )
}
