// UI-3 앱 정보
import { getVersion } from '@tauri-apps/api/app'
import { useEffect, useRef, useState } from 'react'

export default function AboutDialog({ onClose }: { onClose: () => void }) {
  const closeRef = useRef<HTMLButtonElement>(null)
  const [version, setVersion] = useState<string | null>(null)

  // 버전은 설치된 앱이 스스로 밝히는 버전 — tauri.conf.json 한 곳에서 온다
  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null))
  }, [])

  // 열리면 닫기에 포커스
  useEffect(() => {
    closeRef.current?.focus()
  }, [])

  // Esc로 닫는다
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  return (
    <>
      <div className="shade" onClick={onClose} />
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="about-title">
        <h2 id="about-title">포인핸드 대형묘 찾기</h2>
        <p className="ver">{version ? `버전 ${version}` : ' '}</p>
        <p className="lead">포인핸드 공식 앱이 아닙니다.</p>
        <p>
          포인핸드(pawinhand.kr)에 공개된 보호 동물 공고 가운데 몸무게가 기준 이상인 고양이를 모아 보여줍니다. 입양 문의와
          신청은 포인핸드나 각 보호소에서 하세요.
        </p>
        <p className="fine">받아온 목록은 이 PC에 저장하지 않아요. 켤 때마다 새로 받아요.</p>
        <p className="fine">사진은 국가동물보호정보시스템과 포인핸드에서 불러와요.</p>
        <p className="fine">소스 코드 github.com/HoyoungParkme/fourinhand-crawl</p>
        <div className="row">
          <button ref={closeRef} type="button" className="btn primary" onClick={onClose}>
            닫기
          </button>
        </div>
      </div>
    </>
  )
}
