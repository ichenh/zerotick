/** @type {Record<string, import('../i18n.js').LocaleBundle>} */
export const packs = {
  "zh-TW": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "概覽",
      "diagnostics": "診斷",
      "ports": "連接埠",
      "settings": "設定"
    },
    "pages": {
      "overview": {
        "title": "概覽",
        "desc": "即時監控 USB 與藍牙裝置斷線事件"
      },
      "diagnostics": {
        "title": "診斷",
        "desc": "藍牙服務、藍屏追溯與一鍵修復"
      },
      "ports": {
        "title": "連接埠",
        "desc": "檢視並管理本機連接埠佔用"
      },
      "settings": {
        "title": "設定",
        "desc": "監控、歷史記錄與應用程式選項"
      }
    },
    "status": {
      "init": "引擎初始化中…",
      "running": "引擎執行中",
      "failed": "初始化失敗：{err}"
    },
    "tray": {
      "normal": "監控正常",
      "warning": "裝置波動",
      "critical": "異常告警"
    },
    "overview": {
      "hint": "瞬斷事件將自動彈出提醒並高亮顯示",
      "exportJson": "匯出 JSON",
      "exportCsv": "匯出 CSV",
      "clearHistory": "清空歷史",
      "eventsTitle": "裝置事件",
      "eventsMeta": "依時間倒序",
      "empty": "暫無裝置事件，插入或拔出 USB/藍牙裝置後將在此顯示"
    },
    "diag": {
      "bluetooth": {
        "title": "藍牙診斷",
        "scan": "檢測",
        "desc": "檢查藍牙無線電與 bthserv 服務狀態",
        "idle": "點擊「檢測」開始診斷",
        "loading": "檢測中…",
        "ok": "狀態正常",
        "warn": "發現異常",
        "unknown": "未知",
        "radio": "無線電裝置",
        "radioCount": "{n} 個",
        "issues": "問題列表",
        "noIssues": "無異常項"
      },
      "bsod": {
        "title": "藍屏追溯",
        "scan": "掃描",
        "desc": "分析 Minidump 與系統 BugCheck 事件",
        "idle": "點擊「掃描」尋找轉儲檔案",
        "loading": "掃描中…",
        "none": "未發現 Minidump 檔案",
        "recent": "近期藍屏",
        "history": "歷史記錄",
        "bugcheck": "錯誤碼",
        "driver": "驅動程式",
        "dumpPath": "轉儲路徑"
      },
      "repair": {
        "title": "一鍵修復",
        "run": "執行修復",
        "desc": "重啟藍牙與音訊服務，並掃描 USB 選擇性暫停設定",
        "idle": "將重啟 bthserv 與 Audiosrv，並掃描 USB 省電設定",
        "loading": "執行中…",
        "adminHint": "目前非管理員模式，服務重啟可能失敗",
        "adminBanner": "需要管理員權限：請右鍵 ZeroTick →「以系統管理員身分執行」",
        "restarted": "已重啟服務",
        "noneRestarted": "無服務被重啟",
        "failed": "失敗項",
        "usbScan": "USB 省電掃描",
        "noUsbWarn": "未發現啟用省電的 USB 節點"
      }
    },
    "ports": {
      "hint": "開發連接埠 {port} · 可解除 node / vite 等殘留程序",
      "scan": "掃描連接埠",
      "releaseAll": "一鍵解除",
      "releaseAllN": "一鍵解除 ({n})",
      "releaseOne": "解除",
      "scanning": "掃描中…",
      "empty": "點擊「掃描連接埠」檢視本機佔用情況",
      "noListeners": "無本機監聽連接埠",
      "reservedTitle": "Windows TCP 保留區段",
      "category": {
        "releasable": "可解除",
        "in_use": "使用中",
        "inuse": "使用中",
        "time_wait": "殘留連線",
        "system_reserved": "系統保留",
        "free": "可用"
      },
      "message": {
        "time_wait": "TCP 關閉等待中，通常 1–4 分鐘內自動釋放",
        "system_reserved": "連接埠在 Windows 動態保留區段內，請更換開發連接埠",
        "self_app": "ZeroTick 自身佔用，不可結束",
        "protected": "系統/關鍵程序，不可解除",
        "releasable": "開發殘留，可安全結束程序",
        "in_use": "其他程式佔用，結束可能導致異常",
        "unknown": "未知程序佔用，無法判定為殘留",
        "free": "開發伺服器連接埠可用",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "監控",
      "groupData": "歷史記錄",
      "locale": "介面語言",
      "threshold": "瞬斷判定閾值",
      "trayRecovery": "系統匣告警恢復",
      "bluetoothPoll": "藍牙輪詢間隔",
      "historyMax": "歷史保留筆數",
      "timelineMax": "事件列表顯示",
      "nativeNotify": "最小化時顯示通知",
      "launchStartup": "登入時啟動",
      "save": "儲存",
      "groupGeneral": "一般"
    },
    "units": {
      "ms": "毫秒",
      "sec": "秒",
      "count": "筆"
    },
    "events": {
      "transient": "瞬斷",
      "remove": "斷開",
      "arrival": "接入",
      "unknownDevice": "未知裝置",
      "device": "裝置"
    },
    "toast": {
      "saved": "設定已儲存",
      "saveFailed": "儲存失敗：{err}",
      "historyCleared": "歷史記錄已清空",
      "clearFailed": "清空失敗：{err}",
      "exported": "已匯出至 {path}",
      "transient": "瞬斷：{name}（{ms}ms）",
      "disconnected": "裝置斷開：{name}",
      "bluetooth": "藍牙異常：{msg}",
      "bsod": "BSOD 預警：{code}",
      "repairFailed": "修復失敗：{err}",
      "pidKilled": "已結束 PID {pid}",
      "releasedN": "已解除 {n} 個殘留程序",
      "nothingToRelease": "沒有可解除的佔用"
    },
    "spin": {
      "increase": "增加",
      "decrease": "減少"
    },
    "app": {
      "title": "ZeroTick — 系統診斷"
    }
  },
  "ja": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "概要",
      "diagnostics": "診断",
      "ports": "ポート",
      "settings": "設定"
    },
    "pages": {
      "overview": {
        "title": "概要",
        "desc": "USB・Bluetooth の切断をリアルタイム監視"
      },
      "diagnostics": {
        "title": "診断",
        "desc": "Bluetooth サービス、BSOD 追跡、ワンクリック修復"
      },
      "ports": {
        "title": "ポート",
        "desc": "ローカルポートの使用状況を確認・管理"
      },
      "settings": {
        "title": "設定",
        "desc": "監視、履歴、アプリの設定"
      }
    },
    "status": {
      "init": "初期化中…",
      "running": "エンジン稼働中",
      "failed": "初期化に失敗しました：{err}"
    },
    "tray": {
      "normal": "監視正常",
      "warning": "デバイス変動",
      "critical": "アラート"
    },
    "overview": {
      "hint": "瞬断イベントはアラートとハイライトで通知されます",
      "exportJson": "JSON をエクスポート",
      "exportCsv": "CSV をエクスポート",
      "clearHistory": "履歴を消去",
      "eventsTitle": "デバイスイベント",
      "eventsMeta": "新しい順",
      "empty": "イベントはまだありません。USB/Bluetooth デバイスを接続・切断するとここに表示されます。"
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "スキャン",
        "desc": "Bluetooth 無線と bthserv サービスを確認",
        "idle": "「スキャン」をクリックして開始",
        "loading": "スキャン中…",
        "ok": "正常",
        "warn": "問題あり",
        "unknown": "不明",
        "radio": "無線デバイス",
        "radioCount": "{n} 台",
        "issues": "問題",
        "noIssues": "問題なし"
      },
      "bsod": {
        "title": "BSOD 追跡",
        "scan": "スキャン",
        "desc": "Minidump と BugCheck イベントを分析",
        "idle": "「スキャン」をクリックしてダンプを検索",
        "loading": "スキャン中…",
        "none": "Minidump ファイルが見つかりません",
        "recent": "最近の BSOD",
        "history": "履歴",
        "bugcheck": "バグチェック",
        "driver": "ドライバー",
        "dumpPath": "ダンプパス"
      },
      "repair": {
        "title": "ワンクリック修復",
        "run": "修復を実行",
        "desc": "Bluetooth とオーディオサービスを再起動し、USB 選択的サスペンドをスキャン",
        "idle": "bthserv と Audiosrv を再起動し、USB 電源設定をスキャンします",
        "loading": "実行中…",
        "adminHint": "管理者として実行していません — サービス再起動が失敗する場合があります",
        "adminBanner": "管理者権限が必要です：ZeroTick を右クリック → 管理者として実行",
        "restarted": "サービスを再起動しました",
        "noneRestarted": "再起動されたサービスはありません",
        "failed": "失敗項目",
        "usbScan": "USB 電源スキャン",
        "noUsbWarn": "省電力が有効な USB ノードはありません"
      }
    },
    "ports": {
      "hint": "開発ポート {port} · node / vite の残存プロセスを解放",
      "scan": "ポートをスキャン",
      "releaseAll": "すべて解放",
      "releaseAllN": "すべて解放 ({n})",
      "releaseOne": "解放",
      "scanning": "スキャン中…",
      "empty": "「ポートをスキャン」をクリックしてローカル使用状況を表示",
      "noListeners": "ローカル待受ポートなし",
      "reservedTitle": "Windows TCP 除外範囲",
      "category": {
        "releasable": "解放可能",
        "in_use": "使用中",
        "inuse": "使用中",
        "time_wait": "TIME_WAIT",
        "system_reserved": "システム予約",
        "free": "利用可能"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — 通常 1〜4 分で解放されます",
        "system_reserved": "Windows 動的除外範囲内のポート — 開発ポートを変更してください",
        "self_app": "ZeroTick が使用中 — 終了できません",
        "protected": "システム/重要プロセス — 解放できません",
        "releasable": "開発残存 — 安全に終了できます",
        "in_use": "他のアプリが使用中 — 終了すると問題が発生する場合があります",
        "unknown": "不明なプロセス — 残存と判定できません",
        "free": "開発サーバー用に利用可能",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "監視",
      "groupData": "履歴",
      "locale": "表示言語",
      "threshold": "瞬断しきい値",
      "trayRecovery": "トレイアラート復帰",
      "bluetoothPoll": "Bluetooth ポーリング間隔",
      "historyMax": "履歴保持",
      "timelineMax": "イベントリスト表示",
      "nativeNotify": "最小化時に通知を表示",
      "launchStartup": "サインイン時に起動",
      "save": "保存",
      "groupGeneral": "全般"
    },
    "units": {
      "ms": "ms",
      "sec": "秒",
      "count": "件"
    },
    "events": {
      "transient": "瞬断",
      "remove": "切断",
      "arrival": "接続",
      "unknownDevice": "不明なデバイス",
      "device": "デバイス"
    },
    "toast": {
      "saved": "設定を保存しました",
      "saveFailed": "保存に失敗しました：{err}",
      "historyCleared": "履歴を消去しました",
      "clearFailed": "消去に失敗しました：{err}",
      "exported": "{path} にエクスポートしました",
      "transient": "瞬断：{name}（{ms}ms）",
      "disconnected": "切断：{name}",
      "bluetooth": "Bluetooth の問題：{msg}",
      "bsod": "BSOD アラート：{code}",
      "repairFailed": "修復に失敗しました：{err}",
      "pidKilled": "PID {pid} を終了しました",
      "releasedN": "{n} 個のプロセスを解放しました",
      "nothingToRelease": "解放するものはありません"
    },
    "spin": {
      "increase": "増やす",
      "decrease": "減らす"
    },
    "app": {
      "title": "ZeroTick — システム診断"
    }
  },
  "ko": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "개요",
      "diagnostics": "진단",
      "ports": "포트",
      "settings": "설정"
    },
    "pages": {
      "overview": {
        "title": "개요",
        "desc": "USB 및 Bluetooth 연결 해제 실시간 모니터링"
      },
      "diagnostics": {
        "title": "진단",
        "desc": "Bluetooth 서비스, BSOD 추적, 원클릭 복구"
      },
      "ports": {
        "title": "포트",
        "desc": "로컬 포트 사용 현황 확인 및 관리"
      },
      "settings": {
        "title": "설정",
        "desc": "모니터링, 기록 및 앱 옵션"
      }
    },
    "status": {
      "init": "초기화 중…",
      "running": "엔진 실행 중",
      "failed": "초기화 실패: {err}"
    },
    "tray": {
      "normal": "모니터링 정상",
      "warning": "장치 변동",
      "critical": "경고"
    },
    "overview": {
      "hint": "순간 끊김 이벤트는 알림과 강조 표시로 표시됩니다",
      "exportJson": "JSON보내기",
      "exportCsv": "CSV보내기",
      "clearHistory": "기록 지우기",
      "eventsTitle": "장치 이벤트",
      "eventsMeta": "최신순",
      "empty": "아직 이벤트가 없습니다. USB/Bluetooth 장치를 연결하거나 분리하면 여기에 표시됩니다."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "검사",
        "desc": "Bluetooth 무선 및 bthserv 서비스 확인",
        "idle": "「검사」를 클릭하여 시작",
        "loading": "검사 중…",
        "ok": "정상",
        "warn": "문제 발견",
        "unknown": "알 수 없음",
        "radio": "무선 장치",
        "radioCount": "{n}개",
        "issues": "문제",
        "noIssues": "문제 없음"
      },
      "bsod": {
        "title": "BSOD 추적",
        "scan": "검색",
        "desc": "Minidump 및 BugCheck 이벤트 분석",
        "idle": "「검색」을 클릭하여 덤프 파일 찾기",
        "loading": "검색 중…",
        "none": "Minidump 파일을 찾을 수 없음",
        "recent": "최근 BSOD",
        "history": "기록",
        "bugcheck": "버그 체크",
        "driver": "드라이버",
        "dumpPath": "덤프 경로"
      },
      "repair": {
        "title": "원클릭 복구",
        "run": "복구 실행",
        "desc": "Bluetooth 및 오디오 서비스 재시작, USB 선택적 일시 중단 검사",
        "idle": "bthserv 및 Audiosrv를 재시작하고 USB 전원 설정을 검사합니다",
        "loading": "실행 중…",
        "adminHint": "관리자 권한으로 실행 중이 아님 — 서비스 재시작이 실패할 수 있음",
        "adminBanner": "관리자 권한 필요: ZeroTick 우클릭 → 관리자 권한으로 실행",
        "restarted": "서비스 재시작됨",
        "noneRestarted": "재시작된 서비스 없음",
        "failed": "실패 항목",
        "usbScan": "USB 전원 검사",
        "noUsbWarn": "전원 절약이 활성화된 USB 노드 없음"
      }
    },
    "ports": {
      "hint": "개발 포트 {port} · node / vite 잔여 프로세스 해제",
      "scan": "포트 검사",
      "releaseAll": "모두 해제",
      "releaseAllN": "모두 해제 ({n})",
      "releaseOne": "해제",
      "scanning": "검사 중…",
      "empty": "「포트 검사」를 클릭하여 로컬 사용 현황 보기",
      "noListeners": "로컬 수신 포트 없음",
      "reservedTitle": "Windows TCP 제외 범위",
      "category": {
        "releasable": "해제 가능",
        "in_use": "사용 중",
        "inuse": "사용 중",
        "time_wait": "TIME_WAIT",
        "system_reserved": "시스템 예약",
        "free": "사용 가능"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — 보통 1~4분 내 해제됨",
        "system_reserved": "Windows 동적 제외 범위 내 포트 — 개발 포트 변경 필요",
        "self_app": "ZeroTick이 사용 중 — 종료할 수 없음",
        "protected": "시스템/중요 프로세스 — 해제 불가",
        "releasable": "개발 잔여 — 안전하게 종료 가능",
        "in_use": "다른 앱이 사용 중 — 종료 시 문제 발생 가능",
        "unknown": "알 수 없는 프로세스 — 잔여로 분류 불가",
        "free": "개발 서버용 포트 사용 가능",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "모니터링",
      "groupData": "기록",
      "locale": "표시 언어",
      "threshold": "순간 끊김 임계값",
      "trayRecovery": "트레이 경고 복구",
      "bluetoothPoll": "Bluetooth 폴링 간격",
      "historyMax": "기록 보존",
      "timelineMax": "이벤트 목록 표시",
      "nativeNotify": "최소화 시 알림 표시",
      "launchStartup": "로그인 시 시작",
      "save": "저장",
      "groupGeneral": "일반"
    },
    "units": {
      "ms": "ms",
      "sec": "초",
      "count": "개"
    },
    "events": {
      "transient": "순간 끊김",
      "remove": "연결 해제",
      "arrival": "연결",
      "unknownDevice": "알 수 없는 장치",
      "device": "장치"
    },
    "toast": {
      "saved": "설정이 저장되었습니다",
      "saveFailed": "저장 실패: {err}",
      "historyCleared": "기록이 지워졌습니다",
      "clearFailed": "지우기 실패: {err}",
      "exported": "{path}에보냄",
      "transient": "순간 끊김: {name} ({ms}ms)",
      "disconnected": "연결 해제: {name}",
      "bluetooth": "Bluetooth 문제: {msg}",
      "bsod": "BSOD 경고: {code}",
      "repairFailed": "복구 실패: {err}",
      "pidKilled": "PID {pid} 종료됨",
      "releasedN": "{n}개 프로세스 해제됨",
      "nothingToRelease": "해제할 항목 없음"
    },
    "spin": {
      "increase": "늘리기",
      "decrease": "줄이기"
    },
    "app": {
      "title": "ZeroTick — 시스템 진단"
    }
  },
  "de": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Übersicht",
      "diagnostics": "Diagnose",
      "ports": "Ports",
      "settings": "Einstellungen"
    },
    "pages": {
      "overview": {
        "title": "Übersicht",
        "desc": "Echtzeitüberwachung von USB- und Bluetooth-Trennungen"
      },
      "diagnostics": {
        "title": "Diagnose",
        "desc": "Bluetooth-Dienst, BSOD-Nachverfolgung und Ein-Klick-Reparatur"
      },
      "ports": {
        "title": "Ports",
        "desc": "Lokale Portnutzung anzeigen und verwalten"
      },
      "settings": {
        "title": "Einstellungen",
        "desc": "Überwachung, Verlauf und App-Einstellungen"
      }
    },
    "status": {
      "init": "Initialisierung…",
      "running": "Engine läuft",
      "failed": "Initialisierung fehlgeschlagen: {err}"
    },
    "tray": {
      "normal": "Überwachung OK",
      "warning": "Geräteschwankung",
      "critical": "Alarm"
    },
    "overview": {
      "hint": "Kurzzeitige Trennungen lösen Alarme und Hervorhebungen aus",
      "exportJson": "JSON exportieren",
      "exportCsv": "CSV exportieren",
      "clearHistory": "Verlauf löschen",
      "eventsTitle": "Geräteereignisse",
      "eventsMeta": "Neueste zuerst",
      "empty": "Noch keine Ereignisse. Stecken Sie ein USB-/Bluetooth-Gerät ein oder aus, um Aktivität zu sehen."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Scannen",
        "desc": "Bluetooth-Funk und bthserv-Dienst prüfen",
        "idle": "Klicken Sie auf „Scannen“, um zu starten",
        "loading": "Scannen…",
        "ok": "In Ordnung",
        "warn": "Probleme gefunden",
        "unknown": "Unbekannt",
        "radio": "Funkgeräte",
        "radioCount": "{n} Gerät(e)",
        "issues": "Probleme",
        "noIssues": "Keine Probleme"
      },
      "bsod": {
        "title": "BSOD-Nachverfolgung",
        "scan": "Scannen",
        "desc": "Minidump und BugCheck-Ereignisse analysieren",
        "idle": "Klicken Sie auf „Scannen“, um Dump-Dateien zu finden",
        "loading": "Scannen…",
        "none": "Keine Minidump-Dateien gefunden",
        "recent": "Aktuelle BSOD",
        "history": "Verlauf",
        "bugcheck": "Bugcheck",
        "driver": "Treiber",
        "dumpPath": "Dump-Pfad"
      },
      "repair": {
        "title": "Ein-Klick-Reparatur",
        "run": "Reparatur ausführen",
        "desc": "Bluetooth- und Audiodienste neu starten; USB-Selektives Aussetzen scannen",
        "idle": "Startet bthserv & Audiosrv neu und scannt USB-Stromeinstellungen",
        "loading": "Wird ausgeführt…",
        "adminHint": "Nicht als Administrator ausgeführt — Dienstneustart kann fehlschlagen",
        "adminBanner": "Administrator erforderlich: ZeroTick rechtsklicken → Als Administrator ausführen",
        "restarted": "Dienste neu gestartet",
        "noneRestarted": "Keine Dienste neu gestartet",
        "failed": "Fehlgeschlagene Elemente",
        "usbScan": "USB-Stromscan",
        "noUsbWarn": "Keine USB-Knoten mit aktivierter Energieeinsparung"
      }
    },
    "ports": {
      "hint": "Entwicklungsport {port} · node / vite-Überreste freigeben",
      "scan": "Ports scannen",
      "releaseAll": "Alle freigeben",
      "releaseAllN": "Alle freigeben ({n})",
      "releaseOne": "Freigeben",
      "scanning": "Scannen…",
      "empty": "Klicken Sie auf „Ports scannen“, um die lokale Nutzung anzuzeigen",
      "noListeners": "Keine lokalen Listening-Ports",
      "reservedTitle": "Windows TCP-Ausschlussbereiche",
      "category": {
        "releasable": "Freigebbar",
        "in_use": "In Benutzung",
        "inuse": "In Benutzung",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systemreserviert",
        "free": "Verfügbar"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — wird normalerweise in 1–4 Minuten freigegeben",
        "system_reserved": "Port im dynamischen Windows-Ausschlussbereich — Entwicklungsport ändern",
        "self_app": "Von ZeroTick verwendet — kann nicht beendet werden",
        "protected": "System-/kritischer Prozess — kann nicht freigegeben werden",
        "releasable": "Entwicklungsüberrest — sicher beendbar",
        "in_use": "Von anderer App verwendet — Beenden kann Probleme verursachen",
        "unknown": "Unbekannter Prozess — kann nicht als Überrest klassifiziert werden",
        "free": "Port für Entwicklungsserver verfügbar",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Überwachung",
      "groupData": "Verlauf",
      "locale": "Anzeigesprache",
      "threshold": "Kurzzeit-Schwellenwert",
      "trayRecovery": "Tray-Alarm-Wiederherstellung",
      "bluetoothPoll": "Bluetooth-Abfrageintervall",
      "historyMax": "Verlaufsaufbewahrung",
      "timelineMax": "Ereignislistenanzeige",
      "nativeNotify": "Benachrichtigungen bei minimiertem Fenster",
      "launchStartup": "Beim Anmelden starten",
      "save": "Speichern",
      "groupGeneral": "Allgemein"
    },
    "units": {
      "ms": "ms",
      "sec": "Sek.",
      "count": "Einträge"
    },
    "events": {
      "transient": "Kurzzeitig",
      "remove": "Getrennt",
      "arrival": "Verbunden",
      "unknownDevice": "Unbekanntes Gerät",
      "device": "Gerät"
    },
    "toast": {
      "saved": "Einstellungen gespeichert",
      "saveFailed": "Speichern fehlgeschlagen: {err}",
      "historyCleared": "Verlauf gelöscht",
      "clearFailed": "Löschen fehlgeschlagen: {err}",
      "exported": "Exportiert nach {path}",
      "transient": "Kurzzeitig: {name} ({ms}ms)",
      "disconnected": "Getrennt: {name}",
      "bluetooth": "Bluetooth-Problem: {msg}",
      "bsod": "BSOD-Alarm: {code}",
      "repairFailed": "Reparatur fehlgeschlagen: {err}",
      "pidKilled": "PID {pid} beendet",
      "releasedN": "{n} Prozess(e) freigegeben",
      "nothingToRelease": "Nichts freizugeben"
    },
    "spin": {
      "increase": "Erhöhen",
      "decrease": "Verringern"
    },
    "app": {
      "title": "ZeroTick — Systemdiagnose"
    }
  },
  "fr": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Aperçu",
      "diagnostics": "Diagnostic",
      "ports": "Ports",
      "settings": "Paramètres"
    },
    "pages": {
      "overview": {
        "title": "Aperçu",
        "desc": "Surveillance en temps réel des déconnexions USB et Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnostic",
        "desc": "Service Bluetooth, traçage BSOD et réparation en un clic"
      },
      "ports": {
        "title": "Ports",
        "desc": "Afficher et gérer l'utilisation des ports locaux"
      },
      "settings": {
        "title": "Paramètres",
        "desc": "Surveillance, historique et préférences"
      }
    },
    "status": {
      "init": "Initialisation…",
      "running": "Moteur en cours d'exécution",
      "failed": "Échec de l'initialisation : {err}"
    },
    "tray": {
      "normal": "Surveillance OK",
      "warning": "Fluctuation d'appareil",
      "critical": "Alerte"
    },
    "overview": {
      "hint": "Les déconnexions transitoires déclenchent des alertes et des surbrillances",
      "exportJson": "Exporter JSON",
      "exportCsv": "Exporter CSV",
      "clearHistory": "Effacer l'historique",
      "eventsTitle": "Événements d'appareils",
      "eventsMeta": "Plus récents en premier",
      "empty": "Aucun événement pour l'instant. Branchez ou débranchez un appareil USB/Bluetooth pour voir l'activité ici."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Analyser",
        "desc": "Vérifier la radio Bluetooth et le service bthserv",
        "idle": "Cliquez sur Analyser pour commencer",
        "loading": "Analyse…",
        "ok": "Sain",
        "warn": "Problèmes détectés",
        "unknown": "Inconnu",
        "radio": "Appareils radio",
        "radioCount": "{n} appareil(s)",
        "issues": "Problèmes",
        "noIssues": "Aucun problème"
      },
      "bsod": {
        "title": "Traçage BSOD",
        "scan": "Analyser",
        "desc": "Analyser les Minidump et événements BugCheck",
        "idle": "Cliquez sur Analyser pour trouver les fichiers dump",
        "loading": "Analyse…",
        "none": "Aucun fichier Minidump trouvé",
        "recent": "BSOD récent",
        "history": "Historique",
        "bugcheck": "Bug check",
        "driver": "Pilote",
        "dumpPath": "Chemin du dump"
      },
      "repair": {
        "title": "Réparation en un clic",
        "run": "Lancer la réparation",
        "desc": "Redémarrer les services Bluetooth et audio ; analyser la suspension sélective USB",
        "idle": "Redémarre bthserv et Audiosrv et analyse les paramètres d'alimentation USB",
        "loading": "En cours…",
        "adminHint": "Non exécuté en tant qu'administrateur — le redémarrage des services peut échouer",
        "adminBanner": "Administrateur requis : clic droit sur ZeroTick → Exécuter en tant qu'administrateur",
        "restarted": "Services redémarrés",
        "noneRestarted": "Aucun service redémarré",
        "failed": "Éléments échoués",
        "usbScan": "Analyse alimentation USB",
        "noUsbWarn": "Aucun nœud USB avec économie d'énergie activée"
      }
    },
    "ports": {
      "hint": "Port de dev {port} · Libérer les restes node / vite",
      "scan": "Analyser les ports",
      "releaseAll": "Tout libérer",
      "releaseAllN": "Tout libérer ({n})",
      "releaseOne": "Libérer",
      "scanning": "Analyse…",
      "empty": "Cliquez sur Analyser les ports pour voir l'utilisation locale",
      "noListeners": "Aucun port d'écoute local",
      "reservedTitle": "Plages d'exclusion TCP Windows",
      "category": {
        "releasable": "Libérable",
        "in_use": "En cours d'utilisation",
        "inuse": "En cours d'utilisation",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Réservé système",
        "free": "Disponible"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — se libère généralement en 1 à 4 minutes",
        "system_reserved": "Port dans la plage d'exclusion dynamique Windows — changez le port de dev",
        "self_app": "Utilisé par ZeroTick — impossible de terminer",
        "protected": "Processus système/critique — impossible de libérer",
        "releasable": "Reste de dev — peut être terminé en toute sécurité",
        "in_use": "Utilisé par une autre app — la terminaison peut causer des problèmes",
        "unknown": "Processus inconnu — impossible de classer comme reste",
        "free": "Port disponible pour le serveur de développement",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Surveillance",
      "groupData": "Historique",
      "locale": "Langue d'affichage",
      "threshold": "Seuil transitoire",
      "trayRecovery": "Récupération d'alerte plateau",
      "bluetoothPoll": "Intervalle d'interrogation Bluetooth",
      "historyMax": "Rétention de l'historique",
      "timelineMax": "Affichage de la liste d'événements",
      "nativeNotify": "Notifications lorsque minimisé",
      "launchStartup": "Démarrer à la connexion",
      "save": "Enregistrer",
      "groupGeneral": "Général"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "éléments"
    },
    "events": {
      "transient": "Transitoire",
      "remove": "Déconnecté",
      "arrival": "Connecté",
      "unknownDevice": "Appareil inconnu",
      "device": "Appareil"
    },
    "toast": {
      "saved": "Paramètres enregistrés",
      "saveFailed": "Échec de l'enregistrement : {err}",
      "historyCleared": "Historique effacé",
      "clearFailed": "Échec de l'effacement : {err}",
      "exported": "Exporté vers {path}",
      "transient": "Transitoire : {name} ({ms}ms)",
      "disconnected": "Déconnecté : {name}",
      "bluetooth": "Problème Bluetooth : {msg}",
      "bsod": "Alerte BSOD : {code}",
      "repairFailed": "Échec de la réparation : {err}",
      "pidKilled": "PID {pid} terminé",
      "releasedN": "{n} processus libéré(s)",
      "nothingToRelease": "Rien à libérer"
    },
    "spin": {
      "increase": "Augmenter",
      "decrease": "Diminuer"
    },
    "app": {
      "title": "ZeroTick — Diagnostic système"
    }
  },
  "es": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Resumen",
      "diagnostics": "Diagnóstico",
      "ports": "Puertos",
      "settings": "Ajustes"
    },
    "pages": {
      "overview": {
        "title": "Resumen",
        "desc": "Monitoreo en tiempo real de desconexiones USB y Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnóstico",
        "desc": "Servicio Bluetooth, rastreo BSOD y reparación con un clic"
      },
      "ports": {
        "title": "Puertos",
        "desc": "Ver y gestionar el uso de puertos locales"
      },
      "settings": {
        "title": "Ajustes",
        "desc": "Supervisión, historial y preferencias"
      }
    },
    "status": {
      "init": "Inicializando…",
      "running": "Motor en ejecución",
      "failed": "Error de inicialización: {err}"
    },
    "tray": {
      "normal": "Monitoreo OK",
      "warning": "Fluctuación de dispositivo",
      "critical": "Alerta"
    },
    "overview": {
      "hint": "Las desconexiones transitorias activan alertas y resaltados",
      "exportJson": "Exportar JSON",
      "exportCsv": "Exportar CSV",
      "clearHistory": "Borrar historial",
      "eventsTitle": "Eventos de dispositivos",
      "eventsMeta": "Más recientes primero",
      "empty": "Aún no hay eventos. Conecta o desconecta un dispositivo USB/Bluetooth para ver actividad aquí."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Escanear",
        "desc": "Comprobar radio Bluetooth y servicio bthserv",
        "idle": "Haz clic en Escanear para comenzar",
        "loading": "Escaneando…",
        "ok": "Saludable",
        "warn": "Problemas detectados",
        "unknown": "Desconocido",
        "radio": "Dispositivos de radio",
        "radioCount": "{n} dispositivo(s)",
        "issues": "Problemas",
        "noIssues": "Sin problemas"
      },
      "bsod": {
        "title": "Rastreo BSOD",
        "scan": "Escanear",
        "desc": "Analizar Minidump y eventos BugCheck",
        "idle": "Haz clic en Escanear para buscar archivos dump",
        "loading": "Escaneando…",
        "none": "No se encontraron archivos Minidump",
        "recent": "BSOD reciente",
        "history": "Registro histórico",
        "bugcheck": "Bug check",
        "driver": "Controlador",
        "dumpPath": "Ruta del dump"
      },
      "repair": {
        "title": "Reparación con un clic",
        "run": "Ejecutar reparación",
        "desc": "Reiniciar servicios Bluetooth y audio; escanear suspensión selectiva USB",
        "idle": "Reinicia bthserv y Audiosrv y escanea la configuración de energía USB",
        "loading": "Ejecutando…",
        "adminHint": "No se ejecuta como administrador — el reinicio del servicio puede fallar",
        "adminBanner": "Se requiere administrador: clic derecho en ZeroTick → Ejecutar como administrador",
        "restarted": "Servicios reiniciados",
        "noneRestarted": "Ningún servicio reiniciado",
        "failed": "Elementos fallidos",
        "usbScan": "Escaneo de energía USB",
        "noUsbWarn": "No hay nodos USB con ahorro de energía activado"
      }
    },
    "ports": {
      "hint": "Puerto de desarrollo {port} · Liberar restos de node / vite",
      "scan": "Escanear puertos",
      "releaseAll": "Liberar todo",
      "releaseAllN": "Liberar todo ({n})",
      "releaseOne": "Liberar",
      "scanning": "Escaneando…",
      "empty": "Haz clic en Escanear puertos para ver el uso local",
      "noListeners": "Sin puertos de escucha locales",
      "reservedTitle": "Rangos de exclusión TCP de Windows",
      "category": {
        "releasable": "Liberable",
        "in_use": "En uso",
        "inuse": "En uso",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Reservado del sistema",
        "free": "Disponible"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — suele liberarse en 1–4 minutos",
        "system_reserved": "Puerto en rango de exclusión dinámica de Windows — cambia el puerto de desarrollo",
        "self_app": "Usado por ZeroTick — no se puede terminar",
        "protected": "Proceso del sistema/crítico — no se puede liberar",
        "releasable": "Resto de desarrollo — seguro de terminar",
        "in_use": "En uso por otra app — terminar puede causar problemas",
        "unknown": "Proceso desconocido — no se puede clasificar como resto",
        "free": "Puerto disponible para servidor de desarrollo",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Supervisión",
      "groupData": "Historial",
      "locale": "Idioma de visualización",
      "threshold": "Umbral transitorio",
      "trayRecovery": "Recuperación de alerta de bandeja",
      "bluetoothPoll": "Intervalo de sondeo Bluetooth",
      "historyMax": "Retención del historial",
      "timelineMax": "Visualización de lista de eventos",
      "nativeNotify": "Notificaciones al minimizar",
      "launchStartup": "Iniciar al iniciar sesión",
      "save": "Guardar",
      "groupGeneral": "General"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "elementos"
    },
    "events": {
      "transient": "Transitorio",
      "remove": "Desconectado",
      "arrival": "Conectado",
      "unknownDevice": "Dispositivo desconocido",
      "device": "Dispositivo"
    },
    "toast": {
      "saved": "Ajustes guardados",
      "saveFailed": "Error al guardar: {err}",
      "historyCleared": "Historial borrado",
      "clearFailed": "Error al borrar: {err}",
      "exported": "Exportado a {path}",
      "transient": "Transitorio: {name} ({ms}ms)",
      "disconnected": "Desconectado: {name}",
      "bluetooth": "Problema de Bluetooth: {msg}",
      "bsod": "Alerta BSOD: {code}",
      "repairFailed": "Reparación fallida: {err}",
      "pidKilled": "PID {pid} terminado",
      "releasedN": "Liberados {n} proceso(s)",
      "nothingToRelease": "Nada que liberar"
    },
    "spin": {
      "increase": "Aumentar",
      "decrease": "Disminuir"
    },
    "app": {
      "title": "ZeroTick — Diagnóstico del sistema"
    }
  },
  "pt-BR": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Visão geral",
      "diagnostics": "Diagnóstico",
      "ports": "Portas",
      "settings": "Configurações"
    },
    "pages": {
      "overview": {
        "title": "Visão geral",
        "desc": "Monitoramento em tempo real de desconexões USB e Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnóstico",
        "desc": "Serviço Bluetooth, rastreamento BSOD e reparo com um clique"
      },
      "ports": {
        "title": "Portas",
        "desc": "Ver e gerenciar o uso de portas locais"
      },
      "settings": {
        "title": "Configurações",
        "desc": "Monitoramento, histórico e preferências"
      }
    },
    "status": {
      "init": "Inicializando…",
      "running": "Motor em execução",
      "failed": "Falha na inicialização: {err}"
    },
    "tray": {
      "normal": "Monitoramento OK",
      "warning": "Flutuação de dispositivo",
      "critical": "Alerta"
    },
    "overview": {
      "hint": "Desconexões transitórias disparam alertas e destaques",
      "exportJson": "Exportar JSON",
      "exportCsv": "Exportar CSV",
      "clearHistory": "Limpar histórico",
      "eventsTitle": "Eventos de dispositivos",
      "eventsMeta": "Mais recentes primeiro",
      "empty": "Nenhum evento ainda. Conecte ou desconecte um dispositivo USB/Bluetooth para ver atividade aqui."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Verificar",
        "desc": "Verificar rádio Bluetooth e serviço bthserv",
        "idle": "Clique em Verificar para iniciar",
        "loading": "Verificando…",
        "ok": "Saudável",
        "warn": "Problemas encontrados",
        "unknown": "Desconhecido",
        "radio": "Dispositivos de rádio",
        "radioCount": "{n} dispositivo(s)",
        "issues": "Problemas",
        "noIssues": "Sem problemas"
      },
      "bsod": {
        "title": "Rastreamento BSOD",
        "scan": "Verificar",
        "desc": "Analisar Minidump e eventos BugCheck",
        "idle": "Clique em Verificar para encontrar arquivos dump",
        "loading": "Verificando…",
        "none": "Nenhum arquivo Minidump encontrado",
        "recent": "BSOD recente",
        "history": "Registro histórico",
        "bugcheck": "Bug check",
        "driver": "Driver",
        "dumpPath": "Caminho do dump"
      },
      "repair": {
        "title": "Reparo com um clique",
        "run": "Executar reparo",
        "desc": "Reiniciar serviços Bluetooth e áudio; verificar suspensão seletiva USB",
        "idle": "Reinicia bthserv e Audiosrv e verifica configurações de energia USB",
        "loading": "Executando…",
        "adminHint": "Não está executando como administrador — reinício do serviço pode falhar",
        "adminBanner": "Administrador necessário: clique com o botão direito em ZeroTick → Executar como administrador",
        "restarted": "Serviços reiniciados",
        "noneRestarted": "Nenhum serviço reiniciado",
        "failed": "Itens com falha",
        "usbScan": "Verificação de energia USB",
        "noUsbWarn": "Nenhum nó USB com economia de energia ativada"
      }
    },
    "ports": {
      "hint": "Porta de desenvolvimento {port} · Liberar restos de node / vite",
      "scan": "Verificar portas",
      "releaseAll": "Liberar tudo",
      "releaseAllN": "Liberar tudo ({n})",
      "releaseOne": "Liberar",
      "scanning": "Verificando…",
      "empty": "Clique em Verificar portas para ver o uso local",
      "noListeners": "Nenhuma porta de escuta local",
      "reservedTitle": "Intervalos de exclusão TCP do Windows",
      "category": {
        "releasable": "Liberável",
        "in_use": "Em uso",
        "inuse": "Em uso",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Reservado pelo sistema",
        "free": "Disponível"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — geralmente libera em 1–4 minutos",
        "system_reserved": "Porta no intervalo de exclusão dinâmica do Windows — altere a porta de desenvolvimento",
        "self_app": "Usado pelo ZeroTick — não pode ser encerrado",
        "protected": "Processo do sistema/crítico — não pode ser liberado",
        "releasable": "Resto de desenvolvimento — seguro encerrar",
        "in_use": "Em uso por outro app — encerrar pode causar problemas",
        "unknown": "Processo desconhecido — não pode ser classificado como resto",
        "free": "Porta disponível para servidor de desenvolvimento",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitoramento",
      "groupData": "Histórico",
      "locale": "Idioma de exibição",
      "threshold": "Limite transitório",
      "trayRecovery": "Recuperação de alerta da bandeja",
      "bluetoothPoll": "Intervalo de verificação Bluetooth",
      "historyMax": "Retenção do histórico",
      "timelineMax": "Exibição da lista de eventos",
      "nativeNotify": "Notificações quando minimizado",
      "launchStartup": "Iniciar ao entrar",
      "save": "Salvar",
      "groupGeneral": "Geral"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "itens"
    },
    "events": {
      "transient": "Transitório",
      "remove": "Desconectado",
      "arrival": "Conectado",
      "unknownDevice": "Dispositivo desconhecido",
      "device": "Dispositivo"
    },
    "toast": {
      "saved": "Configurações salvas",
      "saveFailed": "Falha ao salvar: {err}",
      "historyCleared": "Histórico limpo",
      "clearFailed": "Falha ao limpar: {err}",
      "exported": "Exportado para {path}",
      "transient": "Transitório: {name} ({ms}ms)",
      "disconnected": "Desconectado: {name}",
      "bluetooth": "Problema de Bluetooth: {msg}",
      "bsod": "Alerta BSOD: {code}",
      "repairFailed": "Reparo falhou: {err}",
      "pidKilled": "PID {pid} encerrado",
      "releasedN": "Liberados {n} processo(s)",
      "nothingToRelease": "Nada para liberar"
    },
    "spin": {
      "increase": "Aumentar",
      "decrease": "Diminuir"
    },
    "app": {
      "title": "ZeroTick — Diagnóstico do sistema"
    }
  },
  "ru": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Обзор",
      "diagnostics": "Диагностика",
      "ports": "Порты",
      "settings": "Настройки"
    },
    "pages": {
      "overview": {
        "title": "Обзор",
        "desc": "Мониторинг отключений USB и Bluetooth в реальном времени"
      },
      "diagnostics": {
        "title": "Диагностика",
        "desc": "Служба Bluetooth, трассировка BSOD и исправление в один клик"
      },
      "ports": {
        "title": "Порты",
        "desc": "Просмотр и управление локальными портами"
      },
      "settings": {
        "title": "Настройки",
        "desc": "Мониторинг, история и параметры"
      }
    },
    "status": {
      "init": "Инициализация…",
      "running": "Движок работает",
      "failed": "Ошибка инициализации: {err}"
    },
    "tray": {
      "normal": "Мониторинг в норме",
      "warning": "Колебания устройства",
      "critical": "Тревога"
    },
    "overview": {
      "hint": "Кратковременные отключения вызывают оповещения и подсветку",
      "exportJson": "Экспорт JSON",
      "exportCsv": "Экспорт CSV",
      "clearHistory": "Очистить историю",
      "eventsTitle": "События устройств",
      "eventsMeta": "Сначала новые",
      "empty": "Событий пока нет. Подключите или отключите USB/Bluetooth-устройство, чтобы увидеть активность здесь."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Сканировать",
        "desc": "Проверить радио Bluetooth и службу bthserv",
        "idle": "Нажмите «Сканировать» для начала",
        "loading": "Сканирование…",
        "ok": "В норме",
        "warn": "Обнаружены проблемы",
        "unknown": "Неизвестно",
        "radio": "Радиоустройства",
        "radioCount": "{n} устр.",
        "issues": "Проблемы",
        "noIssues": "Проблем нет"
      },
      "bsod": {
        "title": "Трассировка BSOD",
        "scan": "Сканировать",
        "desc": "Анализ Minidump и событий BugCheck",
        "idle": "Нажмите «Сканировать» для поиска дампов",
        "loading": "Сканирование…",
        "none": "Файлы Minidump не найдены",
        "recent": "Недавний BSOD",
        "history": "История",
        "bugcheck": "Код ошибки",
        "driver": "Драйвер",
        "dumpPath": "Путь к дампу"
      },
      "repair": {
        "title": "Исправление в один клик",
        "run": "Запустить исправление",
        "desc": "Перезапуск служб Bluetooth и аудио; сканирование выборочной приостановки USB",
        "idle": "Перезапускает bthserv и Audiosrv и сканирует настройки питания USB",
        "loading": "Выполнение…",
        "adminHint": "Не запущено от имени администратора — перезапуск службы может не удаться",
        "adminBanner": "Требуются права администратора: щёлкните правой кнопкой по ZeroTick → Запуск от имени администратора",
        "restarted": "Службы перезапущены",
        "noneRestarted": "Службы не перезапущены",
        "failed": "Неудачные элементы",
        "usbScan": "Сканирование питания USB",
        "noUsbWarn": "Нет узлов USB с включённым энергосбережением"
      }
    },
    "ports": {
      "hint": "Порт разработки {port} · Освободить остатки node / vite",
      "scan": "Сканировать порты",
      "releaseAll": "Освободить все",
      "releaseAllN": "Освободить все ({n})",
      "releaseOne": "Освободить",
      "scanning": "Сканирование…",
      "empty": "Нажмите «Сканировать порты» для просмотра локального использования",
      "noListeners": "Нет локальных портов прослушивания",
      "reservedTitle": "Исключённые диапазоны TCP Windows",
      "category": {
        "releasable": "Можно освободить",
        "in_use": "Используется",
        "inuse": "Используется",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Зарезервировано системой",
        "free": "Доступен"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — обычно освобождается за 1–4 минуты",
        "system_reserved": "Порт в динамическом исключённом диапазоне Windows — смените порт разработки",
        "self_app": "Используется ZeroTick — нельзя завершить",
        "protected": "Системный/критический процесс — нельзя освободить",
        "releasable": "Остаток разработки — безопасно завершить",
        "in_use": "Используется другим приложением — завершение может вызвать проблемы",
        "unknown": "Неизвестный процесс — нельзя классифицировать как остаток",
        "free": "Порт доступен для сервера разработки",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Мониторинг",
      "groupData": "История",
      "locale": "Язык интерфейса",
      "threshold": "Порог кратковременного отключения",
      "trayRecovery": "Восстановление оповещения трея",
      "bluetoothPoll": "Интервал опроса Bluetooth",
      "historyMax": "Хранение истории",
      "timelineMax": "Отображение списка событий",
      "nativeNotify": "Уведомления при свёрнутом окне",
      "launchStartup": "Запуск при входе в систему",
      "save": "Сохранить",
      "groupGeneral": "Общие"
    },
    "units": {
      "ms": "мс",
      "sec": "с",
      "count": "записей"
    },
    "events": {
      "transient": "Кратковременное",
      "remove": "Отключено",
      "arrival": "Подключено",
      "unknownDevice": "Неизвестное устройство",
      "device": "Устройство"
    },
    "toast": {
      "saved": "Настройки сохранены",
      "saveFailed": "Ошибка сохранения: {err}",
      "historyCleared": "История очищена",
      "clearFailed": "Ошибка очистки: {err}",
      "exported": "Экспортировано в {path}",
      "transient": "Кратковременное: {name} ({ms}мс)",
      "disconnected": "Отключено: {name}",
      "bluetooth": "Проблема Bluetooth: {msg}",
      "bsod": "Оповещение BSOD: {code}",
      "repairFailed": "Ошибка исправления: {err}",
      "pidKilled": "Завершён PID {pid}",
      "releasedN": "Освобождено процессов: {n}",
      "nothingToRelease": "Нечего освобождать"
    },
    "spin": {
      "increase": "Увеличить",
      "decrease": "Уменьшить"
    },
    "app": {
      "title": "ZeroTick — Диагностика системы"
    }
  },
  "ar": {
    "meta": {
      "dir": "rtl"
    },
    "nav": {
      "overview": "نظرة عامة",
      "diagnostics": "التشخيص",
      "ports": "المنافذ",
      "settings": "الإعدادات"
    },
    "pages": {
      "overview": {
        "title": "نظرة عامة",
        "desc": "مراقبة فورية لانقطاعات USB وBluetooth"
      },
      "diagnostics": {
        "title": "التشخيص",
        "desc": "خدمة Bluetooth وتتبع BSOD والإصلاح بنقرة واحدة"
      },
      "ports": {
        "title": "المنافذ",
        "desc": "عرض وإدارة استخدام المنافذ المحلية"
      },
      "settings": {
        "title": "الإعدادات",
        "desc": "المراقبة والسجل وتفضيلات التطبيق"
      }
    },
    "status": {
      "init": "جارٍ التهيئة…",
      "running": "المحرك يعمل",
      "failed": "فشل التهيئة: {err}"
    },
    "tray": {
      "normal": "المراقبة سليمة",
      "warning": "تقلب الجهاز",
      "critical": "تنبيه"
    },
    "overview": {
      "hint": "الانقطاعات المؤقتة تُطلق تنبيهات وتمييزًا",
      "exportJson": "تصدير JSON",
      "exportCsv": "تصدير CSV",
      "clearHistory": "مسح السجل",
      "eventsTitle": "أحداث الأجهزة",
      "eventsMeta": "الأحدث أولاً",
      "empty": "لا توجد أحداث بعد. قم بتوصيل أو فصل جهاز USB/Bluetooth لرؤية النشاط هنا."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "فحص",
        "desc": "التحقق من راديو Bluetooth وخدمة bthserv",
        "idle": "انقر فحص للبدء",
        "loading": "جارٍ الفحص…",
        "ok": "سليم",
        "warn": "تم العثور على مشاكل",
        "unknown": "غير معروف",
        "radio": "أجهزة الراديو",
        "radioCount": "{n} جهاز",
        "issues": "المشاكل",
        "noIssues": "لا مشاكل"
      },
      "bsod": {
        "title": "تتبع BSOD",
        "scan": "فحص",
        "desc": "تحليل Minidump وأحداث BugCheck",
        "idle": "انقر فحص للبحث عن ملفات التفريغ",
        "loading": "جارٍ الفحص…",
        "none": "لم يتم العثور على ملفات Minidump",
        "recent": "BSOD حديث",
        "history": "سجل تاريخي",
        "bugcheck": "فحص الخطأ",
        "driver": "برنامج التشغيل",
        "dumpPath": "مسار التفريغ"
      },
      "repair": {
        "title": "إصلاح بنقرة واحدة",
        "run": "تشغيل الإصلاح",
        "desc": "إعادة تشغيل خدمات Bluetooth والصوت؛ فحص التعليق الانتقائي لـ USB",
        "idle": "يعيد تشغيل bthserv وAudiosrv ويفحص إعدادات طاقة USB",
        "loading": "جارٍ التنفيذ…",
        "adminHint": "لا يعمل كمسؤول — قد يفشل إعادة تشغيل الخدمة",
        "adminBanner": "مطلوب مسؤول: انقر بزر الماوس الأيمن على ZeroTick → تشغيل كمسؤول",
        "restarted": "تم إعادة تشغيل الخدمات",
        "noneRestarted": "لم تُعاد تشغيل أي خدمة",
        "failed": "عناصر فاشلة",
        "usbScan": "فحص طاقة USB",
        "noUsbWarn": "لا توجد عقد USB بتوفير الطاقة مفعّل"
      }
    },
    "ports": {
      "hint": "منفذ التطوير {port} · تحرير بقايا node / vite",
      "scan": "فحص المنافذ",
      "releaseAll": "تحرير الكل",
      "releaseAllN": "تحرير الكل ({n})",
      "releaseOne": "تحرير",
      "scanning": "جارٍ الفحص…",
      "empty": "انقر فحص المنافذ لعرض الاستخدام المحلي",
      "noListeners": "لا منافذ استماع محلية",
      "reservedTitle": "نطاقات استبعاد TCP في Windows",
      "category": {
        "releasable": "قابل للتحرير",
        "in_use": "قيد الاستخدام",
        "inuse": "قيد الاستخدام",
        "time_wait": "TIME_WAIT",
        "system_reserved": "محجوز للنظام",
        "free": "متاح"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — يُحرر عادةً خلال 1–4 دقائق",
        "system_reserved": "المنفذ في نطاق الاستبعاد الديناميكي لـ Windows — غيّر منفذ التطوير",
        "self_app": "يستخدمه ZeroTick — لا يمكن إنهاؤه",
        "protected": "عملية نظام/حرجة — لا يمكن تحريرها",
        "releasable": "بقايا تطوير — آمن للإنهاء",
        "in_use": "يستخدمه تطبيق آخر — الإنهاء قد يسبب مشاكل",
        "unknown": "عملية غير معروفة — لا يمكن تصنيفها كبقايا",
        "free": "المنفذ متاح لخادم التطوير",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "المراقبة",
      "groupData": "السجل",
      "locale": "لغة الواجهة",
      "threshold": "عتبة الانقطاع المؤقت",
      "trayRecovery": "استعادة تنبيه منطقة الإعلاعات",
      "bluetoothPoll": "فترة استطلاع Bluetooth",
      "historyMax": "الاحتفاظ بالسجل",
      "timelineMax": "عرض قائمة الأحداث",
      "nativeNotify": "إشعارات عند التصغير",
      "launchStartup": "التشغيل عند تسجيل الدخول",
      "save": "حفظ",
      "groupGeneral": "عام"
    },
    "units": {
      "ms": "مللي ثانية",
      "sec": "ثانية",
      "count": "عناصر"
    },
    "events": {
      "transient": "مؤقت",
      "remove": "منفصل",
      "arrival": "متصل",
      "unknownDevice": "جهاز غير معروف",
      "device": "جهاز"
    },
    "toast": {
      "saved": "تم حفظ الإعدادات",
      "saveFailed": "فشل الحفظ: {err}",
      "historyCleared": "تم مسح السجل",
      "clearFailed": "فشل المسح: {err}",
      "exported": "تم التصدير إلى {path}",
      "transient": "مؤقت: {name} ({ms}مللي ثانية)",
      "disconnected": "منفصل: {name}",
      "bluetooth": "مشكلة Bluetooth: {msg}",
      "bsod": "تنبيه BSOD: {code}",
      "repairFailed": "فشل الإصلاح: {err}",
      "pidKilled": "تم إنهاء PID {pid}",
      "releasedN": "تم تحرير {n} عملية",
      "nothingToRelease": "لا شيء للتحرير"
    },
    "spin": {
      "increase": "زيادة",
      "decrease": "تقليل"
    },
    "app": {
      "title": "ZeroTick — تشخيص النظام"
    }
  },
  "hi": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "अवलोकन",
      "diagnostics": "निदान",
      "ports": "पोर्ट",
      "settings": "सेटिंग्स"
    },
    "pages": {
      "overview": {
        "title": "अवलोकन",
        "desc": "USB और Bluetooth डिस्कनेक्ट की रीयल-टाइम निगरानी"
      },
      "diagnostics": {
        "title": "निदान",
        "desc": "Bluetooth सेवा, BSOD ट्रेस और एक-क्लिक मरम्मत"
      },
      "ports": {
        "title": "पोर्ट",
        "desc": "स्थानीय पोर्ट उपयोग देखें और प्रबंधित करें"
      },
      "settings": {
        "title": "सेटिंग्स",
        "desc": "निगरानी, इतिहास और ऐप विकल्प"
      }
    },
    "status": {
      "init": "प्रारंभ हो रहा है…",
      "running": "इंजन चल रहा है",
      "failed": "प्रारंभ विफल: {err}"
    },
    "tray": {
      "normal": "निगरानी ठीक",
      "warning": "डिवाइस उतार-चढ़ाव",
      "critical": "अलर्ट"
    },
    "overview": {
      "hint": "क्षणिक डिस्कनेक्ट अलर्ट और हाइलाइट ट्रिगर करते हैं",
      "exportJson": "JSON निर्यात",
      "exportCsv": "CSV निर्यात",
      "clearHistory": "इतिहास साफ़ करें",
      "eventsTitle": "डिवाइस इवेंट",
      "eventsMeta": "नवीनतम पहले",
      "empty": "अभी कोई इवेंट नहीं। USB/Bluetooth डिवाइस प्लग या अनप्लग करें।"
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "स्कैन",
        "desc": "Bluetooth रेडियो और bthserv सेवा जाँचें",
        "idle": "शुरू करने के लिए स्कैन पर क्लिक करें",
        "loading": "स्कैन हो रहा है…",
        "ok": "स्वस्थ",
        "warn": "समस्याएँ मिलीं",
        "unknown": "अज्ञात",
        "radio": "रेडियो डिवाइस",
        "radioCount": "{n} डिवाइस",
        "issues": "समस्याएँ",
        "noIssues": "कोई समस्या नहीं"
      },
      "bsod": {
        "title": "BSOD ट्रेस",
        "scan": "स्कैन",
        "desc": "Minidump और BugCheck इवेंट विश्लेषण",
        "idle": "डंप फ़ाइलें खोजने के लिए स्कैन पर क्लिक करें",
        "loading": "स्कैन हो रहा है…",
        "none": "कोई Minidump फ़ाइल नहीं मिली",
        "recent": "हाल का BSOD",
        "history": "ऐतिहासिक रिकॉर्ड",
        "bugcheck": "बग चेक",
        "driver": "ड्राइवर",
        "dumpPath": "डंप पथ"
      },
      "repair": {
        "title": "एक-क्लिक मरम्मत",
        "run": "मरम्मत चलाएँ",
        "desc": "Bluetooth और ऑडियो सेवाएँ पुनः आरंभ; USB चयनात्मक निलंबन स्कैन",
        "idle": "bthserv और Audiosrv पुनः आरंभ करता है और USB पावर सेटिंग्स स्कैन करता है",
        "loading": "चल रहा है…",
        "adminHint": "व्यवस्थापक के रूप में नहीं चल रहा — सेवा पुनः आरंभ विफल हो सकता है",
        "adminBanner": "व्यवस्थापक आवश्यक: ZeroTick पर राइट-क्लिक → व्यवस्थापक के रूप में चलाएँ",
        "restarted": "सेवाएँ पुनः आरंभ",
        "noneRestarted": "कोई सेवा पुनः आरंभ नहीं",
        "failed": "विफल आइटम",
        "usbScan": "USB पावर स्कैन",
        "noUsbWarn": "पावर सेविंग सक्षम USB नोड नहीं"
      }
    },
    "ports": {
      "hint": "डेव पोर्ट {port} · node / vite अवशेष मुक्त करें",
      "scan": "पोर्ट स्कैन",
      "releaseAll": "सभी मुक्त करें",
      "releaseAllN": "सभी मुक्त करें ({n})",
      "releaseOne": "मुक्त करें",
      "scanning": "स्कैन हो रहा है…",
      "empty": "स्थानीय उपयोग देखने के लिए पोर्ट स्कैन पर क्लिक करें",
      "noListeners": "कोई स्थानीय लिसनिंग पोर्ट नहीं",
      "reservedTitle": "Windows TCP बहिष्कृत रेंज",
      "category": {
        "releasable": "मुक्त करने योग्य",
        "in_use": "उपयोग में",
        "inuse": "उपयोग में",
        "time_wait": "TIME_WAIT",
        "system_reserved": "सिस्टम आरक्षित",
        "free": "उपलब्ध"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — आमतौर पर 1–4 मिनट में मुक्त होता है",
        "system_reserved": "Windows गतिशील बहिष्करण रेंज में पोर्ट — डेव पोर्ट बदलें",
        "self_app": "ZeroTick द्वारा उपयोग — समाप्त नहीं कर सकते",
        "protected": "सिस्टम/महत्वपूर्ण प्रक्रिया — मुक्त नहीं कर सकते",
        "releasable": "डेव अवशेष — सुरक्षित रूप से समाप्त",
        "in_use": "अन्य ऐप द्वारा उपयोग — समाप्ति से समस्या हो सकती है",
        "unknown": "अज्ञात प्रक्रिया — अवशेष के रूप में वर्गीकृत नहीं",
        "free": "डेव सर्वर के लिए पोर्ट उपलब्ध",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "निगरानी",
      "groupData": "इतिहास",
      "locale": "इंटरफ़ेस भाषा",
      "threshold": "क्षणिक थ्रेशहोल्ड",
      "trayRecovery": "ट्रे अलर्ट रिकवरी",
      "bluetoothPoll": "Bluetooth पोल अंतराल",
      "historyMax": "इतिहास प्रतिधारण",
      "timelineMax": "इवेंट सूची प्रदर्शन",
      "nativeNotify": "छोटा करने पर सूचनाएँ",
      "launchStartup": "साइन इन पर प्रारंभ करें",
      "save": "सहेजें",
      "groupGeneral": "सामान्य"
    },
    "units": {
      "ms": "मि.से.",
      "sec": "सेकंड",
      "count": "आइटम"
    },
    "events": {
      "transient": "क्षणिक",
      "remove": "डिस्कनेक्ट",
      "arrival": "कनेक्ट",
      "unknownDevice": "अज्ञात डिवाइस",
      "device": "डिवाइस"
    },
    "toast": {
      "saved": "सेटिंग्स सहेजी गईं",
      "saveFailed": "सहेजना विफल: {err}",
      "historyCleared": "इतिहास साफ़",
      "clearFailed": "साफ़ करना विफल: {err}",
      "exported": "{path} में निर्यात",
      "transient": "क्षणिक: {name} ({ms}मि.से.)",
      "disconnected": "डिस्कनेक्ट: {name}",
      "bluetooth": "Bluetooth समस्या: {msg}",
      "bsod": "BSOD अलर्ट: {code}",
      "repairFailed": "मरम्मत विफल: {err}",
      "pidKilled": "PID {pid} समाप्त",
      "releasedN": "{n} प्रक्रिया मुक्त",
      "nothingToRelease": "मुक्त करने के लिए कुछ नहीं"
    },
    "spin": {
      "increase": "बढ़ाएँ",
      "decrease": "घटाएँ"
    },
    "app": {
      "title": "ZeroTick — सिस्टम निदान"
    }
  },
  "it": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Panoramica",
      "diagnostics": "Diagnostica",
      "ports": "Porte",
      "settings": "Impostazioni"
    },
    "pages": {
      "overview": {
        "title": "Panoramica",
        "desc": "Monitoraggio in tempo reale delle disconnessioni USB e Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnostica",
        "desc": "Servizio Bluetooth, tracciamento BSOD e riparazione con un clic"
      },
      "ports": {
        "title": "Porte",
        "desc": "Visualizza e gestisci l'uso delle porte locali"
      },
      "settings": {
        "title": "Impostazioni",
        "desc": "Monitoraggio, cronologia e preferenze"
      }
    },
    "status": {
      "init": "Inizializzazione…",
      "running": "Motore in esecuzione",
      "failed": "Inizializzazione fallita: {err}"
    },
    "tray": {
      "normal": "Monitoraggio OK",
      "warning": "Fluttuazione dispositivo",
      "critical": "Avviso"
    },
    "overview": {
      "hint": "Le disconnessioni transitorie attivano avvisi ed evidenziazioni",
      "exportJson": "Esporta JSON",
      "exportCsv": "Esporta CSV",
      "clearHistory": "Cancella cronologia",
      "eventsTitle": "Eventi dispositivo",
      "eventsMeta": "Più recenti prima",
      "empty": "Nessun evento ancora. Collega o scollega un dispositivo USB/Bluetooth per vedere l'attività qui."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Scansiona",
        "desc": "Verifica radio Bluetooth e servizio bthserv",
        "idle": "Clicca Scansiona per iniziare",
        "loading": "Scansione…",
        "ok": "Sano",
        "warn": "Problemi rilevati",
        "unknown": "Sconosciuto",
        "radio": "Dispositivi radio",
        "radioCount": "{n} dispositivo/i",
        "issues": "Problemi",
        "noIssues": "Nessun problema"
      },
      "bsod": {
        "title": "Tracciamento BSOD",
        "scan": "Scansiona",
        "desc": "Analizza Minidump ed eventi BugCheck",
        "idle": "Clicca Scansiona per trovare file dump",
        "loading": "Scansione…",
        "none": "Nessun file Minidump trovato",
        "recent": "BSOD recente",
        "history": "Registro storico",
        "bugcheck": "Bug check",
        "driver": "Driver",
        "dumpPath": "Percorso dump"
      },
      "repair": {
        "title": "Riparazione con un clic",
        "run": "Esegui riparazione",
        "desc": "Riavvia servizi Bluetooth e audio; scansiona sospensione selettiva USB",
        "idle": "Riavvia bthserv e Audiosrv e scansiona impostazioni alimentazione USB",
        "loading": "In esecuzione…",
        "adminHint": "Non eseguito come amministratore — il riavvio del servizio potrebbe fallire",
        "adminBanner": "Amministratore richiesto: clic destro su ZeroTick → Esegui come amministratore",
        "restarted": "Servizi riavviati",
        "noneRestarted": "Nessun servizio riavviato",
        "failed": "Elementi falliti",
        "usbScan": "Scansione alimentazione USB",
        "noUsbWarn": "Nessun nodo USB con risparmio energetico attivo"
      }
    },
    "ports": {
      "hint": "Porta dev {port} · Libera residui node / vite",
      "scan": "Scansiona porte",
      "releaseAll": "Libera tutto",
      "releaseAllN": "Libera tutto ({n})",
      "releaseOne": "Libera",
      "scanning": "Scansione…",
      "empty": "Clicca Scansiona porte per vedere l'uso locale",
      "noListeners": "Nessuna porta in ascolto locale",
      "reservedTitle": "Intervalli di esclusione TCP Windows",
      "category": {
        "releasable": "Liberabile",
        "in_use": "In uso",
        "inuse": "In uso",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Riservato sistema",
        "free": "Disponibile"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — di solito si libera in 1–4 minuti",
        "system_reserved": "Porta nell'intervallo di esclusione dinamica Windows — cambia porta dev",
        "self_app": "Usato da ZeroTick — impossibile terminare",
        "protected": "Processo di sistema/critico — impossibile liberare",
        "releasable": "Residuo dev — terminabile in sicurezza",
        "in_use": "In uso da altra app — la terminazione può causare problemi",
        "unknown": "Processo sconosciuto — non classificabile come residuo",
        "free": "Porta disponibile per server di sviluppo",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitoraggio",
      "groupData": "Cronologia",
      "locale": "Lingua dell'interfaccia",
      "threshold": "Soglia transitoria",
      "trayRecovery": "Recupero avviso tray",
      "bluetoothPoll": "Intervallo polling Bluetooth",
      "historyMax": "Conservazione cronologia",
      "timelineMax": "Visualizzazione elenco eventi",
      "nativeNotify": "Notifiche quando ridotto a icona",
      "launchStartup": "Avvia all'accesso",
      "save": "Salva",
      "groupGeneral": "Generale"
    },
    "units": {
      "ms": "ms",
      "sec": "sec",
      "count": "elementi"
    },
    "events": {
      "transient": "Transitorio",
      "remove": "Disconnesso",
      "arrival": "Connesso",
      "unknownDevice": "Dispositivo sconosciuto",
      "device": "Dispositivo"
    },
    "toast": {
      "saved": "Impostazioni salvate",
      "saveFailed": "Salvataggio fallito: {err}",
      "historyCleared": "Cronologia cancellata",
      "clearFailed": "Cancellazione fallita: {err}",
      "exported": "Esportato in {path}",
      "transient": "Transitorio: {name} ({ms}ms)",
      "disconnected": "Disconnesso: {name}",
      "bluetooth": "Problema Bluetooth: {msg}",
      "bsod": "Avviso BSOD: {code}",
      "repairFailed": "Riparazione fallita: {err}",
      "pidKilled": "PID {pid} terminato",
      "releasedN": "Liberati {n} processo/i",
      "nothingToRelease": "Niente da liberare"
    },
    "spin": {
      "increase": "Aumenta",
      "decrease": "Diminuisci"
    },
    "app": {
      "title": "ZeroTick — Diagnostica di sistema"
    }
  },
  "nl": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Overzicht",
      "diagnostics": "Diagnose",
      "ports": "Poorten",
      "settings": "Instellingen"
    },
    "pages": {
      "overview": {
        "title": "Overzicht",
        "desc": "Realtime monitoring van USB- en Bluetooth-verbrekingen"
      },
      "diagnostics": {
        "title": "Diagnose",
        "desc": "Bluetooth-service, BSOD-tracering en reparatie met één klik"
      },
      "ports": {
        "title": "Poorten",
        "desc": "Lokaal poortgebruik bekijken en beheren"
      },
      "settings": {
        "title": "Instellingen",
        "desc": "Monitoring, geschiedenis en voorkeuren"
      }
    },
    "status": {
      "init": "Initialiseren…",
      "running": "Engine actief",
      "failed": "Initialisatie mislukt: {err}"
    },
    "tray": {
      "normal": "Monitoring OK",
      "warning": "Apparaatfluctuatie",
      "critical": "Waarschuwing"
    },
    "overview": {
      "hint": "Kortstondige verbrekingen activeren waarschuwingen en markeringen",
      "exportJson": "JSON exporteren",
      "exportCsv": "CSV exporteren",
      "clearHistory": "Geschiedenis wissen",
      "eventsTitle": "Apparaatgebeurtenissen",
      "eventsMeta": "Nieuwste eerst",
      "empty": "Nog geen gebeurtenissen. Sluit een USB-/Bluetooth-apparaat aan of los om activiteit te zien."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Scannen",
        "desc": "Controleer Bluetooth-radio en bthserv-service",
        "idle": "Klik op Scannen om te starten",
        "loading": "Scannen…",
        "ok": "Gezond",
        "warn": "Problemen gevonden",
        "unknown": "Onbekend",
        "radio": "Radioapparaten",
        "radioCount": "{n} apparaat/apparaten",
        "issues": "Problemen",
        "noIssues": "Geen problemen"
      },
      "bsod": {
        "title": "BSOD-tracering",
        "scan": "Scannen",
        "desc": "Analyseer Minidump en BugCheck-gebeurtenissen",
        "idle": "Klik op Scannen om dumpbestanden te vinden",
        "loading": "Scannen…",
        "none": "Geen Minidump-bestanden gevonden",
        "recent": "Recente BSOD",
        "history": "Historisch record",
        "bugcheck": "Bugcheck",
        "driver": "Stuurprogramma",
        "dumpPath": "Dumppad"
      },
      "repair": {
        "title": "Reparatie met één klik",
        "run": "Reparatie uitvoeren",
        "desc": "Herstart Bluetooth- en audioservices; scan USB-selectief opschorten",
        "idle": "Herstart bthserv en Audiosrv en scant USB-stroominstellingen",
        "loading": "Uitvoeren…",
        "adminHint": "Niet als beheerder uitgevoerd — serviceherstart kan mislukken",
        "adminBanner": "Beheerder vereist: rechtsklik op ZeroTick → Uitvoeren als beheerder",
        "restarted": "Services herstart",
        "noneRestarted": "Geen services herstart",
        "failed": "Mislukte items",
        "usbScan": "USB-stroomscan",
        "noUsbWarn": "Geen USB-knooppunten met energiebesparing ingeschakeld"
      }
    },
    "ports": {
      "hint": "Dev-poort {port} · node / vite-resten vrijgeven",
      "scan": "Poorten scannen",
      "releaseAll": "Alles vrijgeven",
      "releaseAllN": "Alles vrijgeven ({n})",
      "releaseOne": "Vrijgeven",
      "scanning": "Scannen…",
      "empty": "Klik op Poorten scannen om lokaal gebruik te bekijken",
      "noListeners": "Geen lokale luisterpoorten",
      "reservedTitle": "Windows TCP-uitsluitingsbereiken",
      "category": {
        "releasable": "Vrij te geven",
        "in_use": "In gebruik",
        "inuse": "In gebruik",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systeemgereserveerd",
        "free": "Beschikbaar"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — wordt meestal binnen 1–4 minuten vrijgegeven",
        "system_reserved": "Poort in dynamisch Windows-uitsluitingsbereik — wijzig dev-poort",
        "self_app": "Gebruikt door ZeroTick — kan niet beëindigen",
        "protected": "Systeem/kritiek proces — kan niet vrijgeven",
        "releasable": "Dev-restant — veilig te beëindigen",
        "in_use": "In gebruik door andere app — beëindigen kan problemen veroorzaken",
        "unknown": "Onbekend proces — kan niet als restant worden geclassificeerd",
        "free": "Poort beschikbaar voor ontwikkelserver",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitoring",
      "groupData": "Geschiedenis",
      "locale": "Weergavetaal",
      "threshold": "Kortstondige drempel",
      "trayRecovery": "Tray-waarschuwing herstel",
      "bluetoothPoll": "Bluetooth-pollinterval",
      "historyMax": "Geschiedenisbehoud",
      "timelineMax": "Gebeurtenislijstweergave",
      "nativeNotify": "Meldingen wanneer geminimaliseerd",
      "launchStartup": "Starten bij aanmelden",
      "save": "Opslaan",
      "groupGeneral": "Algemeen"
    },
    "units": {
      "ms": "ms",
      "sec": "sec",
      "count": "items"
    },
    "events": {
      "transient": "Kortstondig",
      "remove": "Verbroken",
      "arrival": "Verbonden",
      "unknownDevice": "Onbekend apparaat",
      "device": "Apparaat"
    },
    "toast": {
      "saved": "Instellingen opgeslagen",
      "saveFailed": "Opslaan mislukt: {err}",
      "historyCleared": "Geschiedenis gewist",
      "clearFailed": "Wissen mislukt: {err}",
      "exported": "Geëxporteerd naar {path}",
      "transient": "Kortstondig: {name} ({ms}ms)",
      "disconnected": "Verbroken: {name}",
      "bluetooth": "Bluetooth-probleem: {msg}",
      "bsod": "BSOD-waarschuwing: {code}",
      "repairFailed": "Reparatie mislukt: {err}",
      "pidKilled": "PID {pid} beëindigd",
      "releasedN": "{n} proces(sen) vrijgegeven",
      "nothingToRelease": "Niets vrij te geven"
    },
    "spin": {
      "increase": "Verhogen",
      "decrease": "Verlagen"
    },
    "app": {
      "title": "ZeroTick — Systeemdiagnose"
    }
  },
  "pl": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Przegląd",
      "diagnostics": "Diagnostyka",
      "ports": "Porty",
      "settings": "Ustawienia"
    },
    "pages": {
      "overview": {
        "title": "Przegląd",
        "desc": "Monitorowanie rozłączeń USB i Bluetooth w czasie rzeczywistym"
      },
      "diagnostics": {
        "title": "Diagnostyka",
        "desc": "Usługa Bluetooth, śledzenie BSOD i naprawa jednym kliknięciem"
      },
      "ports": {
        "title": "Porty",
        "desc": "Wyświetlaj i zarządzaj lokalnym użyciem portów"
      },
      "settings": {
        "title": "Ustawienia",
        "desc": "Monitorowanie, historia i ustawienia"
      }
    },
    "status": {
      "init": "Inicjalizacja…",
      "running": "Silnik działa",
      "failed": "Inicjalizacja nie powiodła się: {err}"
    },
    "tray": {
      "normal": "Monitorowanie OK",
      "warning": "Wahania urządzenia",
      "critical": "Alert"
    },
    "overview": {
      "hint": "Przejściowe rozłączenia wywołują alerty i podświetlenia",
      "exportJson": "Eksportuj JSON",
      "exportCsv": "Eksportuj CSV",
      "clearHistory": "Wyczyść historię",
      "eventsTitle": "Zdarzenia urządzeń",
      "eventsMeta": "Najnowsze najpierw",
      "empty": "Brak zdarzeń. Podłącz lub odłącz urządzenie USB/Bluetooth, aby zobaczyć aktywność."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Skanuj",
        "desc": "Sprawdź radio Bluetooth i usługę bthserv",
        "idle": "Kliknij Skanuj, aby rozpocząć",
        "loading": "Skanowanie…",
        "ok": "Sprawny",
        "warn": "Znaleziono problemy",
        "unknown": "Nieznany",
        "radio": "Urządzenia radiowe",
        "radioCount": "{n} urządzenie/urządzeń",
        "issues": "Problemy",
        "noIssues": "Brak problemów"
      },
      "bsod": {
        "title": "Śledzenie BSOD",
        "scan": "Skanuj",
        "desc": "Analizuj Minidump i zdarzenia BugCheck",
        "idle": "Kliknij Skanuj, aby znaleźć pliki zrzutu",
        "loading": "Skanowanie…",
        "none": "Nie znaleziono plików Minidump",
        "recent": "Ostatni BSOD",
        "history": "Rejestr historyczny",
        "bugcheck": "Bug check",
        "driver": "Sterownik",
        "dumpPath": "Ścieżka zrzutu"
      },
      "repair": {
        "title": "Naprawa jednym kliknięciem",
        "run": "Uruchom naprawę",
        "desc": "Uruchom ponownie usługi Bluetooth i audio; skanuj selektywne wstrzymanie USB",
        "idle": "Uruchamia ponownie bthserv i Audiosrv oraz skanuje ustawienia zasilania USB",
        "loading": "Wykonywanie…",
        "adminHint": "Nie uruchomiono jako administrator — ponowne uruchomienie usługi może się nie powieść",
        "adminBanner": "Wymagany administrator: kliknij prawym na ZeroTick → Uruchom jako administrator",
        "restarted": "Usługi uruchomione ponownie",
        "noneRestarted": "Żadna usługa nie została uruchomiona ponownie",
        "failed": "Nieudane elementy",
        "usbScan": "Skan zasilania USB",
        "noUsbWarn": "Brak węzłów USB z włączonym oszczędzaniem energii"
      }
    },
    "ports": {
      "hint": "Port deweloperski {port} · Zwolnij pozostałości node / vite",
      "scan": "Skanuj porty",
      "releaseAll": "Zwolnij wszystkie",
      "releaseAllN": "Zwolnij wszystkie ({n})",
      "releaseOne": "Zwolnij",
      "scanning": "Skanowanie…",
      "empty": "Kliknij Skanuj porty, aby zobaczyć lokalne użycie",
      "noListeners": "Brak lokalnych portów nasłuchu",
      "reservedTitle": "Wykluczone zakresy TCP Windows",
      "category": {
        "releasable": "Do zwolnienia",
        "in_use": "W użyciu",
        "inuse": "W użyciu",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Zarezerwowane systemowo",
        "free": "Dostępny"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — zwykle zwalnia się w 1–4 minuty",
        "system_reserved": "Port w dynamicznym zakresie wykluczenia Windows — zmień port deweloperski",
        "self_app": "Używany przez ZeroTick — nie można zakończyć",
        "protected": "Proces systemowy/krytyczny — nie można zwolnić",
        "releasable": "Pozostałość deweloperska — bezpieczne zakończenie",
        "in_use": "Używany przez inną aplikację — zakończenie może powodować problemy",
        "unknown": "Nieznany proces — nie można sklasyfikować jako pozostałość",
        "free": "Port dostępny dla serwera deweloperskiego",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitorowanie",
      "groupData": "Historia",
      "locale": "Język interfejsu",
      "threshold": "Próg przejściowy",
      "trayRecovery": "Odzyskiwanie alertu zasobnika",
      "bluetoothPoll": "Interwał odpytywania Bluetooth",
      "historyMax": "Przechowywanie historii",
      "timelineMax": "Wyświetlanie listy zdarzeń",
      "nativeNotify": "Powiadomienia po zminimalizowaniu",
      "launchStartup": "Uruchamiaj przy logowaniu",
      "save": "Zapisz",
      "groupGeneral": "Ogólne"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "elementów"
    },
    "events": {
      "transient": "Przejściowe",
      "remove": "Rozłączono",
      "arrival": "Połączono",
      "unknownDevice": "Nieznane urządzenie",
      "device": "Urządzenie"
    },
    "toast": {
      "saved": "Ustawienia zapisane",
      "saveFailed": "Zapis nie powiódł się: {err}",
      "historyCleared": "Historia wyczyszczona",
      "clearFailed": "Czyszczenie nie powiodło się: {err}",
      "exported": "Wyeksportowano do {path}",
      "transient": "Przejściowe: {name} ({ms}ms)",
      "disconnected": "Rozłączono: {name}",
      "bluetooth": "Problem Bluetooth: {msg}",
      "bsod": "Alert BSOD: {code}",
      "repairFailed": "Naprawa nie powiodła się: {err}",
      "pidKilled": "Zakończono PID {pid}",
      "releasedN": "Zwolniono {n} proces(ów)",
      "nothingToRelease": "Nic do zwolnienia"
    },
    "spin": {
      "increase": "Zwiększ",
      "decrease": "Zmniejsz"
    },
    "app": {
      "title": "ZeroTick — Diagnostyka systemu"
    }
  },
  "tr": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Genel Bakış",
      "diagnostics": "Tanı",
      "ports": "Bağlantı Noktaları",
      "settings": "Ayarlar"
    },
    "pages": {
      "overview": {
        "title": "Genel Bakış",
        "desc": "USB ve Bluetooth bağlantı kesmelerinin gerçek zamanlı izlenmesi"
      },
      "diagnostics": {
        "title": "Tanı",
        "desc": "Bluetooth hizmeti, BSOD izleme ve tek tıkla onarım"
      },
      "ports": {
        "title": "Bağlantı Noktaları",
        "desc": "Yerel bağlantı noktası kullanımını görüntüle ve yönet"
      },
      "settings": {
        "title": "Ayarlar",
        "desc": "İzleme, geçmiş ve uygulama seçenekleri"
      }
    },
    "status": {
      "init": "Başlatılıyor…",
      "running": "Motor çalışıyor",
      "failed": "Başlatma başarısız: {err}"
    },
    "tray": {
      "normal": "İzleme OK",
      "warning": "Cihaz dalgalanması",
      "critical": "Uyarı"
    },
    "overview": {
      "hint": "Geçici bağlantı kesmeleri uyarı ve vurgulama tetikler",
      "exportJson": "JSON dışa aktar",
      "exportCsv": "CSV dışa aktar",
      "clearHistory": "Geçmişi temizle",
      "eventsTitle": "Cihaz olayları",
      "eventsMeta": "En yeniler önce",
      "empty": "Henüz olay yok. USB/Bluetooth cihaz takın veya çıkarın."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Tara",
        "desc": "Bluetooth radyo ve bthserv hizmetini kontrol et",
        "idle": "Başlamak için Tara'ya tıklayın",
        "loading": "Taranıyor…",
        "ok": "Sağlıklı",
        "warn": "Sorun bulundu",
        "unknown": "Bilinmiyor",
        "radio": "Radyo cihazları",
        "radioCount": "{n} cihaz",
        "issues": "Sorunlar",
        "noIssues": "Sorun yok"
      },
      "bsod": {
        "title": "BSOD izleme",
        "scan": "Tara",
        "desc": "Minidump ve BugCheck olaylarını analiz et",
        "idle": "Döküm dosyalarını bulmak için Tara'ya tıklayın",
        "loading": "Taranıyor…",
        "none": "Minidump dosyası bulunamadı",
        "recent": "Son BSOD",
        "history": "Geçmiş kayıt",
        "bugcheck": "Bug check",
        "driver": "Sürücü",
        "dumpPath": "Döküm yolu"
      },
      "repair": {
        "title": "Tek tıkla onarım",
        "run": "Onarımı çalıştır",
        "desc": "Bluetooth ve ses hizmetlerini yeniden başlat; USB seçici askıya almayı tara",
        "idle": "bthserv ve Audiosrv'yi yeniden başlatır ve USB güç ayarlarını tarar",
        "loading": "Çalışıyor…",
        "adminHint": "Yönetici olarak çalışmıyor — hizmet yeniden başlatma başarısız olabilir",
        "adminBanner": "Yönetici gerekli: ZeroTick'e sağ tık → Yönetici olarak çalıştır",
        "restarted": "Hizmetler yeniden başlatıldı",
        "noneRestarted": "Hiçbir hizmet yeniden başlatılmadı",
        "failed": "Başarısız öğeler",
        "usbScan": "USB güç taraması",
        "noUsbWarn": "Güç tasarrufu etkin USB düğümü yok"
      }
    },
    "ports": {
      "hint": "Geliştirme bağlantı noktası {port} · node / vite kalıntılarını serbest bırak",
      "scan": "Bağlantı noktalarını tara",
      "releaseAll": "Tümünü serbest bırak",
      "releaseAllN": "Tümünü serbest bırak ({n})",
      "releaseOne": "Serbest bırak",
      "scanning": "Taranıyor…",
      "empty": "Yerel kullanımı görmek için Bağlantı noktalarını tara'ya tıklayın",
      "noListeners": "Yerel dinleme bağlantı noktası yok",
      "reservedTitle": "Windows TCP hariç tutma aralıkları",
      "category": {
        "releasable": "Serbest bırakılabilir",
        "in_use": "Kullanımda",
        "inuse": "Kullanımda",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Sistem ayrılmış",
        "free": "Kullanılabilir"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — genellikle 1–4 dakikada serbest bırakılır",
        "system_reserved": "Windows dinamik hariç tutma aralığındaki bağlantı noktası — geliştirme bağlantı noktasını değiştirin",
        "self_app": "ZeroTick tarafından kullanılıyor — sonlandırılamaz",
        "protected": "Sistem/kritik işlem — serbest bırakılamaz",
        "releasable": "Geliştirme kalıntısı — güvenle sonlandırılabilir",
        "in_use": "Başka uygulama tarafından kullanılıyor — sonlandırma sorunlara yol açabilir",
        "unknown": "Bilinmeyen işlem — kalıntı olarak sınıflandırılamaz",
        "free": "Geliştirme sunucusu için bağlantı noktası kullanılabilir",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "İzleme",
      "groupData": "Geçmiş",
      "locale": "Arayüz dili",
      "threshold": "Geçici eşik",
      "trayRecovery": "Tepsi uyarısı kurtarma",
      "bluetoothPoll": "Bluetooth yoklama aralığı",
      "historyMax": "Geçmiş saklama",
      "timelineMax": "Olay listesi görüntüleme",
      "nativeNotify": "Simge durumuna küçültülünce bildirimler",
      "launchStartup": "Oturum açarken başlat",
      "save": "Kaydet",
      "groupGeneral": "Genel"
    },
    "units": {
      "ms": "ms",
      "sec": "sn",
      "count": "öğe"
    },
    "events": {
      "transient": "Geçici",
      "remove": "Bağlantı kesildi",
      "arrival": "Bağlandı",
      "unknownDevice": "Bilinmeyen cihaz",
      "device": "Cihaz"
    },
    "toast": {
      "saved": "Ayarlar kaydedildi",
      "saveFailed": "Kaydetme başarısız: {err}",
      "historyCleared": "Geçmiş temizlendi",
      "clearFailed": "Temizleme başarısız: {err}",
      "exported": "{path} konumuna dışa aktarıldı",
      "transient": "Geçici: {name} ({ms}ms)",
      "disconnected": "Bağlantı kesildi: {name}",
      "bluetooth": "Bluetooth sorunu: {msg}",
      "bsod": "BSOD uyarısı: {code}",
      "repairFailed": "Onarım başarısız: {err}",
      "pidKilled": "PID {pid} sonlandırıldı",
      "releasedN": "{n} işlem serbest bırakıldı",
      "nothingToRelease": "Serbest bırakılacak bir şey yok"
    },
    "spin": {
      "increase": "Artır",
      "decrease": "Azalt"
    },
    "app": {
      "title": "ZeroTick — Sistem teşhisi"
    }
  },
  "vi": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Tổng quan",
      "diagnostics": "Chẩn đoán",
      "ports": "Cổng",
      "settings": "Cài đặt"
    },
    "pages": {
      "overview": {
        "title": "Tổng quan",
        "desc": "Giám sát ngắt kết nối USB và Bluetooth theo thời gian thực"
      },
      "diagnostics": {
        "title": "Chẩn đoán",
        "desc": "Dịch vụ Bluetooth, truy vết BSOD và sửa chữa một cú nhấp"
      },
      "ports": {
        "title": "Cổng",
        "desc": "Xem và quản lý việc sử dụng cổng cục bộ"
      },
      "settings": {
        "title": "Cài đặt",
        "desc": "Giám sát, lịch sử và tùy chọn ứng dụng"
      }
    },
    "status": {
      "init": "Đang khởi tạo…",
      "running": "Động cơ đang chạy",
      "failed": "Khởi tạo thất bại: {err}"
    },
    "tray": {
      "normal": "Giám sát OK",
      "warning": "Dao động thiết bị",
      "critical": "Cảnh báo"
    },
    "overview": {
      "hint": "Ngắt kết nối tạm thời kích hoạt cảnh báo và làm nổi bật",
      "exportJson": "Xuất JSON",
      "exportCsv": "Xuất CSV",
      "clearHistory": "Xóa lịch sử",
      "eventsTitle": "Sự kiện thiết bị",
      "eventsMeta": "Mới nhất trước",
      "empty": "Chưa có sự kiện. Cắm hoặc rút thiết bị USB/Bluetooth để xem hoạt động tại đây."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Quét",
        "desc": "Kiểm tra radio Bluetooth và dịch vụ bthserv",
        "idle": "Nhấp Quét để bắt đầu",
        "loading": "Đang quét…",
        "ok": "Khỏe mạnh",
        "warn": "Phát hiện vấn đề",
        "unknown": "Không rõ",
        "radio": "Thiết bị radio",
        "radioCount": "{n} thiết bị",
        "issues": "Vấn đề",
        "noIssues": "Không có vấn đề"
      },
      "bsod": {
        "title": "Truy vết BSOD",
        "scan": "Quét",
        "desc": "Phân tích Minidump và sự kiện BugCheck",
        "idle": "Nhấp Quét để tìm tệp dump",
        "loading": "Đang quét…",
        "none": "Không tìm thấy tệp Minidump",
        "recent": "BSOD gần đây",
        "history": "Bản ghi lịch sử",
        "bugcheck": "Bug check",
        "driver": "Trình điều khiển",
        "dumpPath": "Đường dẫn dump"
      },
      "repair": {
        "title": "Sửa chữa một cú nhấp",
        "run": "Chạy sửa chữa",
        "desc": "Khởi động lại dịch vụ Bluetooth và âm thanh; quét tạm dừng chọn lọc USB",
        "idle": "Khởi động lại bthserv và Audiosrv và quét cài đặt nguồn USB",
        "loading": "Đang chạy…",
        "adminHint": "Không chạy với quyền quản trị — khởi động lại dịch vụ có thể thất bại",
        "adminBanner": "Cần quyền quản trị: nhấp chuột phải ZeroTick → Chạy với tư cách quản trị viên",
        "restarted": "Đã khởi động lại dịch vụ",
        "noneRestarted": "Không có dịch vụ được khởi động lại",
        "failed": "Mục thất bại",
        "usbScan": "Quét nguồn USB",
        "noUsbWarn": "Không có nút USB bật tiết kiệm năng lượng"
      }
    },
    "ports": {
      "hint": "Cổng dev {port} · Giải phóng tàn dư node / vite",
      "scan": "Quét cổng",
      "releaseAll": "Giải phóng tất cả",
      "releaseAllN": "Giải phóng tất cả ({n})",
      "releaseOne": "Giải phóng",
      "scanning": "Đang quét…",
      "empty": "Nhấp Quét cổng để xem sử dụng cục bộ",
      "noListeners": "Không có cổng lắng nghe cục bộ",
      "reservedTitle": "Phạm vi loại trừ TCP Windows",
      "category": {
        "releasable": "Có thể giải phóng",
        "in_use": "Đang dùng",
        "inuse": "Đang dùng",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Hệ thống dành riêng",
        "free": "Khả dụng"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — thường giải phóng trong 1–4 phút",
        "system_reserved": "Cổng trong phạm vi loại trừ động Windows — đổi cổng dev",
        "self_app": "ZeroTick đang dùng — không thể kết thúc",
        "protected": "Tiến trình hệ thống/quan trọng — không thể giải phóng",
        "releasable": "Tàn dư dev — an toàn khi kết thúc",
        "in_use": "Ứng dụng khác đang dùng — kết thúc có thể gây sự cố",
        "unknown": "Tiến trình không rõ — không thể phân loại là tàn dư",
        "free": "Cổng khả dụng cho máy chủ phát triển",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Giám sát",
      "groupData": "Lịch sử",
      "locale": "Ngôn ngữ giao diện",
      "threshold": "Ngưỡng tạm thời",
      "trayRecovery": "Khôi phục cảnh báo khay",
      "bluetoothPoll": "Khoảng thăm dò Bluetooth",
      "historyMax": "Lưu giữ lịch sử",
      "timelineMax": "Hiển thị danh sách sự kiện",
      "nativeNotify": "Thông báo khi thu nhỏ",
      "launchStartup": "Khởi động khi đăng nhập",
      "save": "Lưu",
      "groupGeneral": "Chung"
    },
    "units": {
      "ms": "ms",
      "sec": "giây",
      "count": "mục"
    },
    "events": {
      "transient": "Tạm thời",
      "remove": "Đã ngắt",
      "arrival": "Đã kết nối",
      "unknownDevice": "Thiết bị không rõ",
      "device": "Thiết bị"
    },
    "toast": {
      "saved": "Đã lưu cài đặt",
      "saveFailed": "Lưu thất bại: {err}",
      "historyCleared": "Đã xóa lịch sử",
      "clearFailed": "Xóa thất bại: {err}",
      "exported": "Đã xuất sang {path}",
      "transient": "Tạm thời: {name} ({ms}ms)",
      "disconnected": "Đã ngắt: {name}",
      "bluetooth": "Vấn đề Bluetooth: {msg}",
      "bsod": "Cảnh báo BSOD: {code}",
      "repairFailed": "Sửa chữa thất bại: {err}",
      "pidKilled": "Đã kết thúc PID {pid}",
      "releasedN": "Đã giải phóng {n} tiến trình",
      "nothingToRelease": "Không có gì để giải phóng"
    },
    "spin": {
      "increase": "Tăng",
      "decrease": "Giảm"
    },
    "app": {
      "title": "ZeroTick — Chẩn đoán hệ thống"
    }
  },
  "th": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "ภาพรวม",
      "diagnostics": "การวินิจฉัย",
      "ports": "พอร์ต",
      "settings": "การตั้งค่า"
    },
    "pages": {
      "overview": {
        "title": "ภาพรวม",
        "desc": "ตรวจสอบการตัดการเชื่อมต่อ USB และ Bluetooth แบบเรียลไทม์"
      },
      "diagnostics": {
        "title": "การวินิจฉัย",
        "desc": "บริการ Bluetooth การติดตาม BSOD และการซ่อมแซมคลิกเดียว"
      },
      "ports": {
        "title": "พอร์ต",
        "desc": "ดูและจัดการการใช้พอร์ตในเครื่อง"
      },
      "settings": {
        "title": "การตั้งค่า",
        "desc": "การตรวจสอบ ประวัติ และการตั้งค่า"
      }
    },
    "status": {
      "init": "กำลังเริ่มต้น…",
      "running": "เอ็นจินกำลังทำงาน",
      "failed": "เริ่มต้นล้มเหลว: {err}"
    },
    "tray": {
      "normal": "การตรวจสอบปกติ",
      "warning": "อุปกรณ์ผันผวน",
      "critical": "แจ้งเตือน"
    },
    "overview": {
      "hint": "การตัดการเชื่อมต่อชั่วคราวจะทริกเกอร์การแจ้งเตือนและไฮไลต์",
      "exportJson": "ส่งออก JSON",
      "exportCsv": "ส่งออก CSV",
      "clearHistory": "ล้างประวัติ",
      "eventsTitle": "เหตุการณ์อุปกรณ์",
      "eventsMeta": "ใหม่สุดก่อน",
      "empty": "ยังไม่มีเหตุการณ์ เสียบหรือถอดอุปกรณ์ USB/Bluetooth เพื่อดูกิจกรรมที่นี่"
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "สแกน",
        "desc": "ตรวจสอบวิทยุ Bluetooth และบริการ bthserv",
        "idle": "คลิกสแกนเพื่อเริ่ม",
        "loading": "กำลังสแกน…",
        "ok": "ปกติ",
        "warn": "พบปัญหา",
        "unknown": "ไม่ทราบ",
        "radio": "อุปกรณ์วิทยุ",
        "radioCount": "{n} อุปกรณ์",
        "issues": "ปัญหา",
        "noIssues": "ไม่มีปัญหา"
      },
      "bsod": {
        "title": "ติดตาม BSOD",
        "scan": "สแกน",
        "desc": "วิเคราะห์ Minidump และเหตุการณ์ BugCheck",
        "idle": "คลิกสแกนเพื่อค้นหาไฟล์ dump",
        "loading": "กำลังสแกน…",
        "none": "ไม่พบไฟล์ Minidump",
        "recent": "BSOD ล่าสุด",
        "history": "บันทึกประวัติ",
        "bugcheck": "Bug check",
        "driver": "ไดรเวอร์",
        "dumpPath": "เส้นทาง dump"
      },
      "repair": {
        "title": "ซ่อมแซมคลิกเดียว",
        "run": "เรียกใช้การซ่อมแซม",
        "desc": "รีสตาร์ทบริการ Bluetooth และเสียง สแกน USB selective suspend",
        "idle": "รีสตาร์ท bthserv และ Audiosrv และสแกนการตั้งค่าพลังงาน USB",
        "loading": "กำลังทำงาน…",
        "adminHint": "ไม่ได้รันในฐานะผู้ดูแลระบบ — การรีสตาร์ทบริการอาจล้มเหลว",
        "adminBanner": "ต้องการผู้ดูแลระบบ: คลิกขวา ZeroTick → รันในฐานะผู้ดูแลระบบ",
        "restarted": "รีสตาร์ทบริการแล้ว",
        "noneRestarted": "ไม่มีบริการถูกรีสตาร์ท",
        "failed": "รายการล้มเหลว",
        "usbScan": "สแกนพลังงาน USB",
        "noUsbWarn": "ไม่มีโหนด USB ที่เปิดประหยัดพลังงาน"
      }
    },
    "ports": {
      "hint": "พอร์ต dev {port} · ปล่อย node / vite ที่ค้างอยู่",
      "scan": "สแกนพอร์ต",
      "releaseAll": "ปล่อยทั้งหมด",
      "releaseAllN": "ปล่อยทั้งหมด ({n})",
      "releaseOne": "ปล่อย",
      "scanning": "กำลังสแกน…",
      "empty": "คลิกสแกนพอร์ตเพื่อดูการใช้งานในเครื่อง",
      "noListeners": "ไม่มีพอร์ตรับฟังในเครื่อง",
      "reservedTitle": "ช่วงยกเว้น TCP ของ Windows",
      "category": {
        "releasable": "ปล่อยได้",
        "in_use": "กำลังใช้",
        "inuse": "กำลังใช้",
        "time_wait": "TIME_WAIT",
        "system_reserved": "สงวนระบบ",
        "free": "ว่าง"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — มักปล่อยภายใน 1–4 นาที",
        "system_reserved": "พอร์ตอยู่ในช่วงยกเว้นแบบไดนามิกของ Windows — เปลี่ยนพอร์ต dev",
        "self_app": "ZeroTick ใช้อยู่ — ไม่สามารถยุติได้",
        "protected": "กระบวนการระบบ/สำคัญ — ไม่สามารถปล่อยได้",
        "releasable": "ค้างจาก dev — ยุติได้อย่างปลอดภัย",
        "in_use": "แอปอื่นใช้อยู่ — การยุติอาจทำให้เกิดปัญหา",
        "unknown": "กระบวนการไม่ทราบ — ไม่สามารถจัดประเภทเป็นค้างได้",
        "free": "พอร์ตพร้อมสำหรับเซิร์ฟเวอร์พัฒนา",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "การตรวจสอบ",
      "groupData": "ประวัติ",
      "locale": "ภาษาส่วนติดต่อ",
      "threshold": "เกณฑ์ชั่วคราว",
      "trayRecovery": "กู้คืนการแจ้งเตือนถาด",
      "bluetoothPoll": "ช่วงเวลาโพล Bluetooth",
      "historyMax": "เก็บประวัติ",
      "timelineMax": "แสดงรายการเหตุการณ์",
      "nativeNotify": "แจ้งเตือนเมื่อย่อหน้าต่าง",
      "launchStartup": "เริ่มเมื่อลงชื่อเข้าใช้",
      "save": "บันทึก",
      "groupGeneral": "ทั่วไป"
    },
    "units": {
      "ms": "มิลลิวิ",
      "sec": "วินาที",
      "count": "รายการ"
    },
    "events": {
      "transient": "ชั่วคราว",
      "remove": "ตัดการเชื่อมต่อ",
      "arrival": "เชื่อมต่อ",
      "unknownDevice": "อุปกรณ์ไม่ทราบ",
      "device": "อุปกรณ์"
    },
    "toast": {
      "saved": "บันทึกการตั้งค่าแล้ว",
      "saveFailed": "บันทึกล้มเหลว: {err}",
      "historyCleared": "ล้างประวัติแล้ว",
      "clearFailed": "ล้างล้มเหลว: {err}",
      "exported": "ส่งออกไปยัง {path}",
      "transient": "ชั่วคราว: {name} ({ms}มิลลิวิ)",
      "disconnected": "ตัดการเชื่อมต่อ: {name}",
      "bluetooth": "ปัญหา Bluetooth: {msg}",
      "bsod": "แจ้งเตือน BSOD: {code}",
      "repairFailed": "ซ่อมแซมล้มเหลว: {err}",
      "pidKilled": "ยุติ PID {pid} แล้ว",
      "releasedN": "ปล่อย {n} กระบวนการ",
      "nothingToRelease": "ไม่มีอะไรให้ปล่อย"
    },
    "spin": {
      "increase": "เพิ่ม",
      "decrease": "ลด"
    },
    "app": {
      "title": "ZeroTick — การวินิจฉัยระบบ"
    }
  },
  "id": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Ikhtisar",
      "diagnostics": "Diagnostik",
      "ports": "Port",
      "settings": "Pengaturan"
    },
    "pages": {
      "overview": {
        "title": "Ikhtisar",
        "desc": "Pemantauan pemutusan USB dan Bluetooth secara real-time"
      },
      "diagnostics": {
        "title": "Diagnostik",
        "desc": "Layanan Bluetooth, pelacakan BSOD, dan perbaikan satu klik"
      },
      "ports": {
        "title": "Port",
        "desc": "Lihat dan kelola penggunaan port lokal"
      },
      "settings": {
        "title": "Pengaturan",
        "desc": "Pemantauan, riwayat, dan preferensi"
      }
    },
    "status": {
      "init": "Menginisialisasi…",
      "running": "Mesin berjalan",
      "failed": "Inisialisasi gagal: {err}"
    },
    "tray": {
      "normal": "Pemantauan OK",
      "warning": "Fluktuasi perangkat",
      "critical": "Peringatan"
    },
    "overview": {
      "hint": "Pemutusan sementara memicu peringatan dan sorotan",
      "exportJson": "Ekspor JSON",
      "exportCsv": "Ekspor CSV",
      "clearHistory": "Hapus riwayat",
      "eventsTitle": "Peristiwa perangkat",
      "eventsMeta": "Terbaru dulu",
      "empty": "Belum ada peristiwa. Colok atau cabut perangkat USB/Bluetooth untuk melihat aktivitas di sini."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Pindai",
        "desc": "Periksa radio Bluetooth dan layanan bthserv",
        "idle": "Klik Pindai untuk memulai",
        "loading": "Memindai…",
        "ok": "Sehat",
        "warn": "Masalah ditemukan",
        "unknown": "Tidak diketahui",
        "radio": "Perangkat radio",
        "radioCount": "{n} perangkat",
        "issues": "Masalah",
        "noIssues": "Tidak ada masalah"
      },
      "bsod": {
        "title": "Pelacakan BSOD",
        "scan": "Pindai",
        "desc": "Analisis Minidump dan peristiwa BugCheck",
        "idle": "Klik Pindai untuk menemukan file dump",
        "loading": "Memindai…",
        "none": "Tidak ada file Minidump ditemukan",
        "recent": "BSOD terbaru",
        "history": "Catatan historis",
        "bugcheck": "Bug check",
        "driver": "Driver",
        "dumpPath": "Jalur dump"
      },
      "repair": {
        "title": "Perbaikan satu klik",
        "run": "Jalankan perbaikan",
        "desc": "Mulai ulang layanan Bluetooth dan audio; pindai suspend selektif USB",
        "idle": "Memulai ulang bthserv & Audiosrv dan memindai pengaturan daya USB",
        "loading": "Menjalankan…",
        "adminHint": "Tidak berjalan sebagai administrator — mulai ulang layanan mungkin gagal",
        "adminBanner": "Administrator diperlukan: klik kanan ZeroTick → Jalankan sebagai administrator",
        "restarted": "Layanan dimulai ulang",
        "noneRestarted": "Tidak ada layanan dimulai ulang",
        "failed": "Item gagal",
        "usbScan": "Pemindaian daya USB",
        "noUsbWarn": "Tidak ada node USB dengan penghemat daya aktif"
      }
    },
    "ports": {
      "hint": "Port dev {port} · Lepaskan sisa node / vite",
      "scan": "Pindai port",
      "releaseAll": "Lepaskan semua",
      "releaseAllN": "Lepaskan semua ({n})",
      "releaseOne": "Lepaskan",
      "scanning": "Memindai…",
      "empty": "Klik Pindai port untuk melihat penggunaan lokal",
      "noListeners": "Tidak ada port mendengarkan lokal",
      "reservedTitle": "Rentang pengecualian TCP Windows",
      "category": {
        "releasable": "Dapat dilepaskan",
        "in_use": "Sedang digunakan",
        "inuse": "Sedang digunakan",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Dipesan sistem",
        "free": "Tersedia"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — biasanya terlepas dalam 1–4 menit",
        "system_reserved": "Port dalam rentang pengecualian dinamis Windows — ubah port dev",
        "self_app": "Digunakan ZeroTick — tidak dapat dihentikan",
        "protected": "Proses sistem/kritis — tidak dapat dilepaskan",
        "releasable": "Sisa dev — aman dihentikan",
        "in_use": "Digunakan aplikasi lain — menghentikan dapat menyebabkan masalah",
        "unknown": "Proses tidak diketahui — tidak dapat diklasifikasikan sebagai sisa",
        "free": "Port tersedia untuk server pengembangan",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Pemantauan",
      "groupData": "Riwayat",
      "locale": "Bahasa antarmuka",
      "threshold": "Ambang sementara",
      "trayRecovery": "Pemulihan peringatan baki",
      "bluetoothPoll": "Interval polling Bluetooth",
      "historyMax": "Retensi riwayat",
      "timelineMax": "Tampilan daftar peristiwa",
      "nativeNotify": "Notifikasi saat diminimalkan",
      "launchStartup": "Mulai saat masuk",
      "save": "Simpan",
      "groupGeneral": "Umum"
    },
    "units": {
      "ms": "ms",
      "sec": "dtk",
      "count": "item"
    },
    "events": {
      "transient": "Sementara",
      "remove": "Terputus",
      "arrival": "Terhubung",
      "unknownDevice": "Perangkat tidak diketahui",
      "device": "Perangkat"
    },
    "toast": {
      "saved": "Pengaturan disimpan",
      "saveFailed": "Gagal menyimpan: {err}",
      "historyCleared": "Riwayat dihapus",
      "clearFailed": "Gagal menghapus: {err}",
      "exported": "Diekspor ke {path}",
      "transient": "Sementara: {name} ({ms}ms)",
      "disconnected": "Terputus: {name}",
      "bluetooth": "Masalah Bluetooth: {msg}",
      "bsod": "Peringatan BSOD: {code}",
      "repairFailed": "Perbaikan gagal: {err}",
      "pidKilled": "PID {pid} dihentikan",
      "releasedN": "Melepaskan {n} proses",
      "nothingToRelease": "Tidak ada yang dilepaskan"
    },
    "spin": {
      "increase": "Tambah",
      "decrease": "Kurangi"
    },
    "app": {
      "title": "ZeroTick — Diagnostik sistem"
    }
  },
  "cs": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Přehled",
      "diagnostics": "Diagnostika",
      "ports": "Porty",
      "settings": "Nastavení"
    },
    "pages": {
      "overview": {
        "title": "Přehled",
        "desc": "Monitorování odpojení USB a Bluetooth v reálném čase"
      },
      "diagnostics": {
        "title": "Diagnostika",
        "desc": "Služba Bluetooth, sledování BSOD a oprava jedním kliknutím"
      },
      "ports": {
        "title": "Porty",
        "desc": "Zobrazit a spravovat místní využití portů"
      },
      "settings": {
        "title": "Nastavení",
        "desc": "Monitorování, historie a předvolby"
      }
    },
    "status": {
      "init": "Inicializace…",
      "running": "Engine běží",
      "failed": "Inicializace selhala: {err}"
    },
    "tray": {
      "normal": "Monitorování OK",
      "warning": "Kolísání zařízení",
      "critical": "Výstraha"
    },
    "overview": {
      "hint": "Přechodná odpojení spouštějí výstrahy a zvýraznění",
      "exportJson": "Exportovat JSON",
      "exportCsv": "Exportovat CSV",
      "clearHistory": "Vymazat historii",
      "eventsTitle": "Události zařízení",
      "eventsMeta": "Nejnovější první",
      "empty": "Zatím žádné události. Připojte nebo odpojte USB/Bluetooth zařízení."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Skenovat",
        "desc": "Zkontrolovat Bluetooth rádio a službu bthserv",
        "idle": "Klikněte na Skenovat pro začátek",
        "loading": "Skenování…",
        "ok": "V pořádku",
        "warn": "Nalezeny problémy",
        "unknown": "Neznámé",
        "radio": "Rádiová zařízení",
        "radioCount": "{n} zařízení",
        "issues": "Problémy",
        "noIssues": "Žádné problémy"
      },
      "bsod": {
        "title": "Sledování BSOD",
        "scan": "Skenovat",
        "desc": "Analyzovat Minidump a události BugCheck",
        "idle": "Klikněte na Skenovat pro nalezení dump souborů",
        "loading": "Skenování…",
        "none": "Nenalezeny žádné Minidump soubory",
        "recent": "Nedávný BSOD",
        "history": "Historický záznam",
        "bugcheck": "Bug check",
        "driver": "Ovladač",
        "dumpPath": "Cesta k dumpu"
      },
      "repair": {
        "title": "Oprava jedním kliknutím",
        "run": "Spustit opravu",
        "desc": "Restartovat služby Bluetooth a zvuku; skenovat selektivní pozastavení USB",
        "idle": "Restartuje bthserv a Audiosrv a skenuje nastavení napájení USB",
        "loading": "Probíhá…",
        "adminHint": "Neběží jako správce — restart služby může selhat",
        "adminBanner": "Vyžadován správce: pravý klik na ZeroTick → Spustit jako správce",
        "restarted": "Služby restartovány",
        "noneRestarted": "Žádná služba nerestartována",
        "failed": "Neúspěšné položky",
        "usbScan": "Sken napájení USB",
        "noUsbWarn": "Žádné USB uzly s úsporou energie"
      }
    },
    "ports": {
      "hint": "Dev port {port} · Uvolnit zbytky node / vite",
      "scan": "Skenovat porty",
      "releaseAll": "Uvolnit vše",
      "releaseAllN": "Uvolnit vše ({n})",
      "releaseOne": "Uvolnit",
      "scanning": "Skenování…",
      "empty": "Klikněte na Skenovat porty pro zobrazení místního využití",
      "noListeners": "Žádné místní naslouchací porty",
      "reservedTitle": "Vyloučené rozsahy TCP Windows",
      "category": {
        "releasable": "Uvolnitelné",
        "in_use": "Používáno",
        "inuse": "Používáno",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systémově rezervováno",
        "free": "Dostupné"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — obvykle se uvolní za 1–4 minuty",
        "system_reserved": "Port v dynamickém vyloučeném rozsahu Windows — změňte dev port",
        "self_app": "Používá ZeroTick — nelze ukončit",
        "protected": "Systémový/kritický proces — nelze uvolnit",
        "releasable": "Dev zbytek — bezpečně ukončitelné",
        "in_use": "Používá jiná aplikace — ukončení může způsobit problémy",
        "unknown": "Neznámý proces — nelze klasifikovat jako zbytek",
        "free": "Port dostupný pro vývojový server",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitorování",
      "groupData": "Historie",
      "locale": "Jazyk rozhraní",
      "threshold": "Prah přechodného odpojení",
      "trayRecovery": "Obnova tray výstrahy",
      "bluetoothPoll": "Interval dotazování Bluetooth",
      "historyMax": "Uchování historie",
      "timelineMax": "Zobrazení seznamu událostí",
      "nativeNotify": "Oznámení při minimalizaci",
      "launchStartup": "Spustit při přihlášení",
      "save": "Uložit",
      "groupGeneral": "Obecné"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "položek"
    },
    "events": {
      "transient": "Přechodné",
      "remove": "Odpojeno",
      "arrival": "Připojeno",
      "unknownDevice": "Neznámé zařízení",
      "device": "Zařízení"
    },
    "toast": {
      "saved": "Nastavení uloženo",
      "saveFailed": "Uložení selhalo: {err}",
      "historyCleared": "Historie vymazána",
      "clearFailed": "Vymazání selhalo: {err}",
      "exported": "Exportováno do {path}",
      "transient": "Přechodné: {name} ({ms}ms)",
      "disconnected": "Odpojeno: {name}",
      "bluetooth": "Problém Bluetooth: {msg}",
      "bsod": "Výstraha BSOD: {code}",
      "repairFailed": "Oprava selhala: {err}",
      "pidKilled": "Ukončen PID {pid}",
      "releasedN": "Uvolněno {n} procesů",
      "nothingToRelease": "Nic k uvolnění"
    },
    "spin": {
      "increase": "Zvýšit",
      "decrease": "Snížit"
    },
    "app": {
      "title": "ZeroTick — Diagnostika systému"
    }
  },
  "da": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Oversigt",
      "diagnostics": "Diagnostik",
      "ports": "Porte",
      "settings": "Indstillinger"
    },
    "pages": {
      "overview": {
        "title": "Oversigt",
        "desc": "Realtidsovervågning af USB- og Bluetooth-afbrydelser"
      },
      "diagnostics": {
        "title": "Diagnostik",
        "desc": "Bluetooth-tjeneste, BSOD-sporing og reparation med ét klik"
      },
      "ports": {
        "title": "Porte",
        "desc": "Vis og administrer lokal portbrug"
      },
      "settings": {
        "title": "Indstillinger",
        "desc": "Overvågning, historik og indstillinger"
      }
    },
    "status": {
      "init": "Initialiserer…",
      "running": "Motor kører",
      "failed": "Initialisering mislykkedes: {err}"
    },
    "tray": {
      "normal": "Overvågning OK",
      "warning": "Enhedsfluktuation",
      "critical": "Advarsel"
    },
    "overview": {
      "hint": "Midlertidige afbrydelser udløser advarsler og fremhævning",
      "exportJson": "Eksporter JSON",
      "exportCsv": "Eksporter CSV",
      "clearHistory": "Ryd historik",
      "eventsTitle": "Enhedshændelser",
      "eventsMeta": "Nyeste først",
      "empty": "Ingen hændelser endnu. Tilslut eller frakobl en USB-/Bluetooth-enhed for at se aktivitet her."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Scan",
        "desc": "Tjek Bluetooth-radio og bthserv-tjeneste",
        "idle": "Klik Scan for at starte",
        "loading": "Scanner…",
        "ok": "Sund",
        "warn": "Problemer fundet",
        "unknown": "Ukendt",
        "radio": "Radioenheder",
        "radioCount": "{n} enhed(er)",
        "issues": "Problemer",
        "noIssues": "Ingen problemer"
      },
      "bsod": {
        "title": "BSOD-sporing",
        "scan": "Scan",
        "desc": "Analyser Minidump og BugCheck-hændelser",
        "idle": "Klik Scan for at finde dump-filer",
        "loading": "Scanner…",
        "none": "Ingen Minidump-filer fundet",
        "recent": "Seneste BSOD",
        "history": "Historisk post",
        "bugcheck": "Bugcheck",
        "driver": "Driver",
        "dumpPath": "Dump-sti"
      },
      "repair": {
        "title": "Reparation med ét klik",
        "run": "Kør reparation",
        "desc": "Genstart Bluetooth- og lydtjenester; scan USB-selektiv suspendering",
        "idle": "Genstarter bthserv og Audiosrv og scanner USB-strømindstillinger",
        "loading": "Kører…",
        "adminHint": "Kører ikke som administrator — tjenestegenstart kan mislykkes",
        "adminBanner": "Administrator påkrævet: højreklik på ZeroTick → Kør som administrator",
        "restarted": "Tjenester genstartet",
        "noneRestarted": "Ingen tjenester genstartet",
        "failed": "Mislykkede elementer",
        "usbScan": "USB-strømscan",
        "noUsbWarn": "Ingen USB-noder med strømbesparelse aktiveret"
      }
    },
    "ports": {
      "hint": "Dev-port {port} · Frigiv node / vite-rester",
      "scan": "Scan porte",
      "releaseAll": "Frigiv alle",
      "releaseAllN": "Frigiv alle ({n})",
      "releaseOne": "Frigiv",
      "scanning": "Scanner…",
      "empty": "Klik Scan porte for at se lokal brug",
      "noListeners": "Ingen lokale lytteporte",
      "reservedTitle": "Windows TCP-udelukkelsesområder",
      "category": {
        "releasable": "Kan frigives",
        "in_use": "I brug",
        "inuse": "I brug",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systemreserveret",
        "free": "Tilgængelig"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — frigives normalt inden for 1–4 minutter",
        "system_reserved": "Port i Windows dynamisk udelukkelsesområde — skift dev-port",
        "self_app": "Bruges af ZeroTick — kan ikke afsluttes",
        "protected": "System/kritisk proces — kan ikke frigives",
        "releasable": "Dev-rester — sikkert at afslutte",
        "in_use": "I brug af anden app — afslutning kan forårsage problemer",
        "unknown": "Ukendt proces — kan ikke klassificeres som rest",
        "free": "Port tilgængelig for udviklingsserver",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Overvågning",
      "groupData": "Historik",
      "locale": "Visningssprog",
      "threshold": "Midlertidig tærskel",
      "trayRecovery": "Bakkeadvarsel-gendannelse",
      "bluetoothPoll": "Bluetooth-afstemningsinterval",
      "historyMax": "Historikopbevaring",
      "timelineMax": "Hændelseslistevisning",
      "nativeNotify": "Meddelelser når minimeret",
      "launchStartup": "Start ved logon",
      "save": "Gem",
      "groupGeneral": "Generelt"
    },
    "units": {
      "ms": "ms",
      "sec": "sek",
      "count": "elementer"
    },
    "events": {
      "transient": "Midlertidig",
      "remove": "Afbrudt",
      "arrival": "Forbundet",
      "unknownDevice": "Ukendt enhed",
      "device": "Enhed"
    },
    "toast": {
      "saved": "Indstillinger gemt",
      "saveFailed": "Gem mislykkedes: {err}",
      "historyCleared": "Historik ryddet",
      "clearFailed": "Ryd mislykkedes: {err}",
      "exported": "Eksporteret til {path}",
      "transient": "Midlertidig: {name} ({ms}ms)",
      "disconnected": "Afbrudt: {name}",
      "bluetooth": "Bluetooth-problem: {msg}",
      "bsod": "BSOD-advarsel: {code}",
      "repairFailed": "Reparation mislykkedes: {err}",
      "pidKilled": "PID {pid} afsluttet",
      "releasedN": "Frigivet {n} proces(sen)",
      "nothingToRelease": "Intet at frigive"
    },
    "spin": {
      "increase": "Forøg",
      "decrease": "Formindsk"
    },
    "app": {
      "title": "ZeroTick — Systemdiagnosticering"
    }
  },
  "fi": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Yleiskatsaus",
      "diagnostics": "Diagnostiikka",
      "ports": "Portit",
      "settings": "Asetukset"
    },
    "pages": {
      "overview": {
        "title": "Yleiskatsaus",
        "desc": "USB- ja Bluetooth-yhteyksien katkeamisen reaaliaikainen seuranta"
      },
      "diagnostics": {
        "title": "Diagnostiikka",
        "desc": "Bluetooth-palvelu, BSOD-jäljitys ja yhden napsautuksen korjaus"
      },
      "ports": {
        "title": "Portit",
        "desc": "Näytä ja hallitse paikallista porttikäyttöä"
      },
      "settings": {
        "title": "Asetukset",
        "desc": "Seuranta, historia ja asetukset"
      }
    },
    "status": {
      "init": "Alustetaan…",
      "running": "Moottori käynnissä",
      "failed": "Alustus epäonnistui: {err}"
    },
    "tray": {
      "normal": "Seuranta OK",
      "warning": "Laitteen vaihtelu",
      "critical": "Hälytys"
    },
    "overview": {
      "hint": "Hetkelliset katkeamiset laukaisevat hälytykset ja korostukset",
      "exportJson": "Vie JSON",
      "exportCsv": "Vie CSV",
      "clearHistory": "Tyhjennä historia",
      "eventsTitle": "Laite­tapahtumat",
      "eventsMeta": "Uusimmat ensin",
      "empty": "Ei tapahtumia vielä. Kytke USB-/Bluetooth-laite päälle tai irti nähdäksesi toiminnan täällä."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Skannaa",
        "desc": "Tarkista Bluetooth-radio ja bthserv-palvelu",
        "idle": "Napsauta Skannaa aloittaaksesi",
        "loading": "Skannataan…",
        "ok": "Kunnossa",
        "warn": "Ongelmia löydetty",
        "unknown": "Tuntematon",
        "radio": "Radiolaitteet",
        "radioCount": "{n} laitetta",
        "issues": "Ongelmat",
        "noIssues": "Ei ongelmia"
      },
      "bsod": {
        "title": "BSOD-jäljitys",
        "scan": "Skannaa",
        "desc": "Analysoi Minidump ja BugCheck-tapahtumat",
        "idle": "Napsauta Skannaa etsiäksesi dump-tiedostoja",
        "loading": "Skannataan…",
        "none": "Minidump-tiedostoja ei löytynyt",
        "recent": "Viimeisin BSOD",
        "history": "Historiatietue",
        "bugcheck": "Bug check",
        "driver": "Ajuri",
        "dumpPath": "Dump-polku"
      },
      "repair": {
        "title": "Yhden napsautuksen korjaus",
        "run": "Suorita korjaus",
        "desc": "Käynnistä Bluetooth- ja äänipalvelut uudelleen; skannaa USB-valikoiva keskeytys",
        "idle": "Käynnistää bthservin ja Audiosrvin uudelleen ja skannaa USB-virta-asetukset",
        "loading": "Suoritetaan…",
        "adminHint": "Ei suoriteta järjestelmänvalvojana — palvelun uudelleenkäynnistys voi epäonnistua",
        "adminBanner": "Järjestelmänvalvoja vaaditaan: napsauta ZeroTickiä hiiren oikealla → Suorita järjestelmänvalvojana",
        "restarted": "Palvelut käynnistetty uudelleen",
        "noneRestarted": "Palveluita ei käynnistetty uudelleen",
        "failed": "Epäonnistuneet kohteet",
        "usbScan": "USB-virranskannaus",
        "noUsbWarn": "Ei USB-solmuja virransäästöllä käytössä"
      }
    },
    "ports": {
      "hint": "Dev-portti {port} · Vapauta node / vite-jäännökset",
      "scan": "Skannaa portit",
      "releaseAll": "Vapauta kaikki",
      "releaseAllN": "Vapauta kaikki ({n})",
      "releaseOne": "Vapauta",
      "scanning": "Skannataan…",
      "empty": "Napsauta Skannaa portit nähdäksesi paikallisen käytön",
      "noListeners": "Ei paikallisia kuunteluportteja",
      "reservedTitle": "Windowsin TCP-poissulkemisalueet",
      "category": {
        "releasable": "Vapautettavissa",
        "in_use": "Käytössä",
        "inuse": "Käytössä",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Järjestelmävarattu",
        "free": "Vapaa"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — vapautuu yleensä 1–4 minuutissa",
        "system_reserved": "Portti Windowsin dynaamisessa poissulkemisalueessa — vaihda dev-portti",
        "self_app": "ZeroTick käyttää — ei voi lopettaa",
        "protected": "Järjestelmä-/kriittinen prosessi — ei voi vapauttaa",
        "releasable": "Dev-jäännös — turvallista lopettaa",
        "in_use": "Toinen sovellus käyttää — lopettaminen voi aiheuttaa ongelmia",
        "unknown": "Tuntematon prosessi — ei voida luokitella jäännökseksi",
        "free": "Portti käytettävissä kehityspalvelimelle",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Seuranta",
      "groupData": "Historia",
      "locale": "Käyttöliittymän kieli",
      "threshold": "Hetkellinen kynnys",
      "trayRecovery": "Ilmoitusalueen hälytyksen palautus",
      "bluetoothPoll": "Bluetooth-kyselyväli",
      "historyMax": "Historian säilytys",
      "timelineMax": "Tapahtumalistan näyttö",
      "nativeNotify": "Ilmoitukset pienennettynä",
      "launchStartup": "Käynnistä kirjautuessa",
      "save": "Tallenna",
      "groupGeneral": "Yleiset"
    },
    "units": {
      "ms": "ms",
      "sec": "s",
      "count": "kohdetta"
    },
    "events": {
      "transient": "Hetkellinen",
      "remove": "Katkaistu",
      "arrival": "Yhdistetty",
      "unknownDevice": "Tuntematon laite",
      "device": "Laite"
    },
    "toast": {
      "saved": "Asetukset tallennettu",
      "saveFailed": "Tallennus epäonnistui: {err}",
      "historyCleared": "Historia tyhjennetty",
      "clearFailed": "Tyhjennys epäonnistui: {err}",
      "exported": "Viety kohteeseen {path}",
      "transient": "Hetkellinen: {name} ({ms}ms)",
      "disconnected": "Katkaistu: {name}",
      "bluetooth": "Bluetooth-ongelma: {msg}",
      "bsod": "BSOD-hälytys: {code}",
      "repairFailed": "Korjaus epäonnistui: {err}",
      "pidKilled": "PID {pid} lopetettu",
      "releasedN": "Vapautettu {n} prosessia",
      "nothingToRelease": "Ei vapautettavaa"
    },
    "spin": {
      "increase": "Lisää",
      "decrease": "Vähennä"
    },
    "app": {
      "title": "ZeroTick — Järjestelmän diagnostiikka"
    }
  },
  "nb": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Oversikt",
      "diagnostics": "Diagnostikk",
      "ports": "Porter",
      "settings": "Innstillinger"
    },
    "pages": {
      "overview": {
        "title": "Oversikt",
        "desc": "Sanntidsovervåking av USB- og Bluetooth-frakoblinger"
      },
      "diagnostics": {
        "title": "Diagnostikk",
        "desc": "Bluetooth-tjeneste, BSOD-sporing og reparasjon med ett klikk"
      },
      "ports": {
        "title": "Porter",
        "desc": "Vis og administrer lokal portbruk"
      },
      "settings": {
        "title": "Innstillinger",
        "desc": "Overvåking, historikk og innstillinger"
      }
    },
    "status": {
      "init": "Initialiserer…",
      "running": "Motor kjører",
      "failed": "Initialisering mislyktes: {err}"
    },
    "tray": {
      "normal": "Overvåking OK",
      "warning": "Enhetsfluktuasjon",
      "critical": "Varsel"
    },
    "overview": {
      "hint": "Midlertidige frakoblinger utløser varsler og utheving",
      "exportJson": "Eksporter JSON",
      "exportCsv": "Eksporter CSV",
      "clearHistory": "Tøm historikk",
      "eventsTitle": "Enhetshendelser",
      "eventsMeta": "Nyeste først",
      "empty": "Ingen hendelser ennå. Koble til eller fra en USB-/Bluetooth-enhet for å se aktivitet her."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Skann",
        "desc": "Sjekk Bluetooth-radio og bthserv-tjeneste",
        "idle": "Klikk Skann for å starte",
        "loading": "Skanner…",
        "ok": "Frisk",
        "warn": "Problemer funnet",
        "unknown": "Ukjent",
        "radio": "Radioenheter",
        "radioCount": "{n} enhet(er)",
        "issues": "Problemer",
        "noIssues": "Ingen problemer"
      },
      "bsod": {
        "title": "BSOD-sporing",
        "scan": "Skann",
        "desc": "Analyser Minidump og BugCheck-hendelser",
        "idle": "Klikk Skann for å finne dump-filer",
        "loading": "Skanner…",
        "none": "Ingen Minidump-filer funnet",
        "recent": "Nylig BSOD",
        "history": "Historisk post",
        "bugcheck": "Bugcheck",
        "driver": "Driver",
        "dumpPath": "Dump-sti"
      },
      "repair": {
        "title": "Reparasjon med ett klikk",
        "run": "Kjør reparasjon",
        "desc": "Start Bluetooth- og lydtjenester på nytt; skann USB-selektiv suspendering",
        "idle": "Starter bthserv og Audiosrv på nytt og skanner USB-strøminnstillinger",
        "loading": "Kjører…",
        "adminHint": "Kjører ikke som administrator — tjenestenyttstart kan mislykkes",
        "adminBanner": "Administrator kreves: høyreklikk på ZeroTick → Kjør som administrator",
        "restarted": "Tjenester startet på nytt",
        "noneRestarted": "Ingen tjenester startet på nytt",
        "failed": "Mislykkede elementer",
        "usbScan": "USB-strømskann",
        "noUsbWarn": "Ingen USB-noder med strømsparing aktivert"
      }
    },
    "ports": {
      "hint": "Dev-port {port} · Frigjør node / vite-rester",
      "scan": "Skann porter",
      "releaseAll": "Frigjør alle",
      "releaseAllN": "Frigjør alle ({n})",
      "releaseOne": "Frigjør",
      "scanning": "Skanner…",
      "empty": "Klikk Skann porter for å se lokal bruk",
      "noListeners": "Ingen lokale lytteporter",
      "reservedTitle": "Windows TCP-eksklusjonsområder",
      "category": {
        "releasable": "Kan frigjøres",
        "in_use": "I bruk",
        "inuse": "I bruk",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systemreservert",
        "free": "Tilgjengelig"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — frigjøres vanligvis innen 1–4 minutter",
        "system_reserved": "Port i Windows dynamisk eksklusjonsområde — endre dev-port",
        "self_app": "Brukes av ZeroTick — kan ikke avsluttes",
        "protected": "System/kritisk prosess — kan ikke frigjøres",
        "releasable": "Dev-rester — trygt å avslutte",
        "in_use": "I bruk av annen app — avslutning kan forårsake problemer",
        "unknown": "Ukjent prosess — kan ikke klassifiseres som rest",
        "free": "Port tilgjengelig for utviklingsserver",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Overvåking",
      "groupData": "Historikk",
      "locale": "Visningsspråk",
      "threshold": "Midlertidig terskel",
      "trayRecovery": "Systemkurv-varselgjenoppretting",
      "bluetoothPoll": "Bluetooth-avstemningsintervall",
      "historyMax": "Historikkbevaring",
      "timelineMax": "Hendelseslistevisning",
      "nativeNotify": "Varsler når minimert",
      "launchStartup": "Start ved pålogging",
      "save": "Lagre",
      "groupGeneral": "Generelt"
    },
    "units": {
      "ms": "ms",
      "sec": "sek",
      "count": "elementer"
    },
    "events": {
      "transient": "Midlertidig",
      "remove": "Frakoblet",
      "arrival": "Tilkoblet",
      "unknownDevice": "Ukjent enhet",
      "device": "Enhet"
    },
    "toast": {
      "saved": "Innstillinger lagret",
      "saveFailed": "Lagring mislyktes: {err}",
      "historyCleared": "Historikk tømt",
      "clearFailed": "Tømming mislyktes: {err}",
      "exported": "Eksportert til {path}",
      "transient": "Midlertidig: {name} ({ms}ms)",
      "disconnected": "Frakoblet: {name}",
      "bluetooth": "Bluetooth-problem: {msg}",
      "bsod": "BSOD-varsel: {code}",
      "repairFailed": "Reparasjon mislyktes: {err}",
      "pidKilled": "PID {pid} avsluttet",
      "releasedN": "Frigjort {n} prosess(er)",
      "nothingToRelease": "Ingenting å frigjøre"
    },
    "spin": {
      "increase": "Øk",
      "decrease": "Reduser"
    },
    "app": {
      "title": "ZeroTick — Systemdiagnose"
    }
  },
  "sv": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Översikt",
      "diagnostics": "Diagnostik",
      "ports": "Portar",
      "settings": "Inställningar"
    },
    "pages": {
      "overview": {
        "title": "Översikt",
        "desc": "Realtidsövervakning av USB- och Bluetooth-avkopplingar"
      },
      "diagnostics": {
        "title": "Diagnostik",
        "desc": "Bluetooth-tjänst, BSOD-spårning och reparation med ett klick"
      },
      "ports": {
        "title": "Portar",
        "desc": "Visa och hantera lokal portanvändning"
      },
      "settings": {
        "title": "Inställningar",
        "desc": "Övervakning, historik och inställningar"
      }
    },
    "status": {
      "init": "Initierar…",
      "running": "Motor körs",
      "failed": "Initiering misslyckades: {err}"
    },
    "tray": {
      "normal": "Övervakning OK",
      "warning": "Enhetsfluktuation",
      "critical": "Varning"
    },
    "overview": {
      "hint": "Tillfälliga avkopplingar utlöser varningar och markeringar",
      "exportJson": "Exportera JSON",
      "exportCsv": "Exportera CSV",
      "clearHistory": "Rensa historik",
      "eventsTitle": "Enhetshändelser",
      "eventsMeta": "Nyaste först",
      "empty": "Inga händelser ännu. Anslut eller koppla från en USB-/Bluetooth-enhet för att se aktivitet här."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Skanna",
        "desc": "Kontrollera Bluetooth-radio och bthserv-tjänst",
        "idle": "Klicka Skanna för att starta",
        "loading": "Skannar…",
        "ok": "Frisk",
        "warn": "Problem hittade",
        "unknown": "Okänd",
        "radio": "Radioenheter",
        "radioCount": "{n} enhet(er)",
        "issues": "Problem",
        "noIssues": "Inga problem"
      },
      "bsod": {
        "title": "BSOD-spårning",
        "scan": "Skanna",
        "desc": "Analysera Minidump och BugCheck-händelser",
        "idle": "Klicka Skanna för att hitta dumpfiler",
        "loading": "Skannar…",
        "none": "Inga Minidump-filer hittades",
        "recent": "Senaste BSOD",
        "history": "Historiskt register",
        "bugcheck": "Bugcheck",
        "driver": "Drivrutin",
        "dumpPath": "Dumpsökväg"
      },
      "repair": {
        "title": "Reparation med ett klick",
        "run": "Kör reparation",
        "desc": "Starta om Bluetooth- och ljudtjänster; skanna USB-selektiv avstängning",
        "idle": "Startar om bthserv och Audiosrv och skannar USB-ströminställningar",
        "loading": "Kör…",
        "adminHint": "Körs inte som administratör — tjänstomstart kan misslyckas",
        "adminBanner": "Administratör krävs: högerklicka på ZeroTick → Kör som administratör",
        "restarted": "Tjänster omstartade",
        "noneRestarted": "Inga tjänster omstartade",
        "failed": "Misslyckade objekt",
        "usbScan": "USB-strömskannning",
        "noUsbWarn": "Inga USB-noder med strömsparning aktiverad"
      }
    },
    "ports": {
      "hint": "Dev-port {port} · Frigör node / vite-rester",
      "scan": "Skanna portar",
      "releaseAll": "Frigör alla",
      "releaseAllN": "Frigör alla ({n})",
      "releaseOne": "Frigör",
      "scanning": "Skannar…",
      "empty": "Klicka Skanna portar för att se lokal användning",
      "noListeners": "Inga lokala lyssningsportar",
      "reservedTitle": "Windows TCP-undantagsområden",
      "category": {
        "releasable": "Kan frigöras",
        "in_use": "I bruk",
        "inuse": "I bruk",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Systemreserverad",
        "free": "Tillgänglig"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — frigörs vanligtvis inom 1–4 minuter",
        "system_reserved": "Port i Windows dynamiskt undantagsområde — byt dev-port",
        "self_app": "Används av ZeroTick — kan inte avslutas",
        "protected": "System/kritisk process — kan inte frigöras",
        "releasable": "Dev-rester — säkert att avsluta",
        "in_use": "Används av annan app — avslutning kan orsaka problem",
        "unknown": "Okänd process — kan inte klassificeras som rest",
        "free": "Port tillgänglig för utvecklingsserver",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Övervakning",
      "groupData": "Historik",
      "locale": "Visningsspråk",
      "threshold": "Tillfällig tröskel",
      "trayRecovery": "Systemfältsvarningsåterställning",
      "bluetoothPoll": "Bluetooth-avfrågningsintervall",
      "historyMax": "Historikbevarande",
      "timelineMax": "Händelselistevisning",
      "nativeNotify": "Aviseringar vid minimering",
      "launchStartup": "Starta vid inloggning",
      "save": "Spara",
      "groupGeneral": "Allmänt"
    },
    "units": {
      "ms": "ms",
      "sec": "sek",
      "count": "objekt"
    },
    "events": {
      "transient": "Tillfällig",
      "remove": "Frånkopplad",
      "arrival": "Ansluten",
      "unknownDevice": "Okänd enhet",
      "device": "Enhet"
    },
    "toast": {
      "saved": "Inställningar sparade",
      "saveFailed": "Sparning misslyckades: {err}",
      "historyCleared": "Historik rensad",
      "clearFailed": "Rensning misslyckades: {err}",
      "exported": "Exporterad till {path}",
      "transient": "Tillfällig: {name} ({ms}ms)",
      "disconnected": "Frånkopplad: {name}",
      "bluetooth": "Bluetooth-problem: {msg}",
      "bsod": "BSOD-varning: {code}",
      "repairFailed": "Reparation misslyckades: {err}",
      "pidKilled": "PID {pid} avslutad",
      "releasedN": "Frigjort {n} process(er)",
      "nothingToRelease": "Inget att frigöra"
    },
    "spin": {
      "increase": "Öka",
      "decrease": "Minska"
    },
    "app": {
      "title": "ZeroTick — Systemdiagnostik"
    }
  },
  "uk": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Огляд",
      "diagnostics": "Діагностика",
      "ports": "Порти",
      "settings": "Налаштування"
    },
    "pages": {
      "overview": {
        "title": "Огляд",
        "desc": "Моніторинг відключень USB і Bluetooth у реальному часі"
      },
      "diagnostics": {
        "title": "Діагностика",
        "desc": "Служба Bluetooth, відстеження BSOD і виправлення в один клік"
      },
      "ports": {
        "title": "Порти",
        "desc": "Перегляд і керування локальним використанням портів"
      },
      "settings": {
        "title": "Налаштування",
        "desc": "Моніторинг, історія та параметри"
      }
    },
    "status": {
      "init": "Ініціалізація…",
      "running": "Двигун працює",
      "failed": "Помилка ініціалізації: {err}"
    },
    "tray": {
      "normal": "Моніторинг у нормі",
      "warning": "Коливання пристрою",
      "critical": "Тривога"
    },
    "overview": {
      "hint": "Короткочасні відключення викликають сповіщення та підсвічування",
      "exportJson": "Експорт JSON",
      "exportCsv": "Експорт CSV",
      "clearHistory": "Очистити історію",
      "eventsTitle": "Події пристроїв",
      "eventsMeta": "Спочатку нові",
      "empty": "Подій поки немає. Підключіть або відключіть USB/Bluetooth-пристрій, щоб побачити активність тут."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Сканувати",
        "desc": "Перевірити радіо Bluetooth і службу bthserv",
        "idle": "Натисніть «Сканувати» для початку",
        "loading": "Сканування…",
        "ok": "У нормі",
        "warn": "Виявлено проблеми",
        "unknown": "Невідомо",
        "radio": "Радіопристрої",
        "radioCount": "{n} пристр.",
        "issues": "Проблеми",
        "noIssues": "Проблем немає"
      },
      "bsod": {
        "title": "Відстеження BSOD",
        "scan": "Сканувати",
        "desc": "Аналіз Minidump і подій BugCheck",
        "idle": "Натисніть «Сканувати» для пошуку дампів",
        "loading": "Сканування…",
        "none": "Файли Minidump не знайдено",
        "recent": "Нещодавній BSOD",
        "history": "Історія",
        "bugcheck": "Код помилки",
        "driver": "Драйвер",
        "dumpPath": "Шлях до дампу"
      },
      "repair": {
        "title": "Виправлення в один клік",
        "run": "Запустити виправлення",
        "desc": "Перезапуск служб Bluetooth і аудіо; сканування вибіркового призупинення USB",
        "idle": "Перезапускає bthserv і Audiosrv і сканує налаштування живлення USB",
        "loading": "Виконання…",
        "adminHint": "Не запущено від імені адміністратора — перезапуск служби може не вдатися",
        "adminBanner": "Потрібні права адміністратора: клацніть правою кнопкою по ZeroTick → Запуск від імені адміністратора",
        "restarted": "Служби перезапущено",
        "noneRestarted": "Служби не перезапущено",
        "failed": "Невдалі елементи",
        "usbScan": "Сканування живлення USB",
        "noUsbWarn": "Немає вузлів USB з увімкненим енергозбереженням"
      }
    },
    "ports": {
      "hint": "Порт розробки {port} · Звільнити залишки node / vite",
      "scan": "Сканувати порти",
      "releaseAll": "Звільнити всі",
      "releaseAllN": "Звільнити всі ({n})",
      "releaseOne": "Звільнити",
      "scanning": "Сканування…",
      "empty": "Натисніть «Сканувати порти» для перегляду локального використання",
      "noListeners": "Немає локальних портів прослуховування",
      "reservedTitle": "Виключені діапазони TCP Windows",
      "category": {
        "releasable": "Можна звільнити",
        "in_use": "Використовується",
        "inuse": "Використовується",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Зарезервовано системою",
        "free": "Доступний"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — зазвичай звільняється за 1–4 хвилини",
        "system_reserved": "Порт у динамічному виключеному діапазоні Windows — змініть порт розробки",
        "self_app": "Використовує ZeroTick — не можна завершити",
        "protected": "Системний/критичний процес — не можна звільнити",
        "releasable": "Залишок розробки — безпечно завершити",
        "in_use": "Використовує інша програма — завершення може спричинити проблеми",
        "unknown": "Невідомий процес — не можна класифікувати як залишок",
        "free": "Порт доступний для сервера розробки",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Моніторинг",
      "groupData": "Історія",
      "locale": "Мова інтерфейсу",
      "threshold": "Поріг короткочасного відключення",
      "trayRecovery": "Відновлення сповіщення трею",
      "bluetoothPoll": "Інтервал опитування Bluetooth",
      "historyMax": "Зберігання історії",
      "timelineMax": "Відображення списку подій",
      "nativeNotify": "Сповіщення у згорнутому вікні",
      "launchStartup": "Запуск під час входу",
      "save": "Зберегти",
      "groupGeneral": "Загальні"
    },
    "units": {
      "ms": "мс",
      "sec": "с",
      "count": "записів"
    },
    "events": {
      "transient": "Короткочасне",
      "remove": "Відключено",
      "arrival": "Підключено",
      "unknownDevice": "Невідомий пристрій",
      "device": "Пристрій"
    },
    "toast": {
      "saved": "Налаштування збережено",
      "saveFailed": "Помилка збереження: {err}",
      "historyCleared": "Історію очищено",
      "clearFailed": "Помилка очищення: {err}",
      "exported": "Експортовано в {path}",
      "transient": "Короткочасне: {name} ({ms}мс)",
      "disconnected": "Відключено: {name}",
      "bluetooth": "Проблема Bluetooth: {msg}",
      "bsod": "Сповіщення BSOD: {code}",
      "repairFailed": "Помилка виправлення: {err}",
      "pidKilled": "Завершено PID {pid}",
      "releasedN": "Звільнено процесів: {n}",
      "nothingToRelease": "Немає чого звільняти"
    },
    "spin": {
      "increase": "Збільшити",
      "decrease": "Зменшити"
    },
    "app": {
      "title": "ZeroTick — Діагностика системи"
    }
  },
  "he": {
    "meta": {
      "dir": "rtl"
    },
    "nav": {
      "overview": "סקירה",
      "diagnostics": "אבחון",
      "ports": "יציאות",
      "settings": "הגדרות"
    },
    "pages": {
      "overview": {
        "title": "סקירה",
        "desc": "ניטור בזמן אמת של ניתוקי USB ו-Bluetooth"
      },
      "diagnostics": {
        "title": "אבחון",
        "desc": "שירות Bluetooth, מעקב BSOD ותיקון בלחיצה אחת"
      },
      "ports": {
        "title": "יציאות",
        "desc": "הצגה וניהול של שימוש ביציאות מקומיות"
      },
      "settings": {
        "title": "הגדרות",
        "desc": "ניטור, היסטוריה והעדפות"
      }
    },
    "status": {
      "init": "מאתחל…",
      "running": "מנוע פועל",
      "failed": "האתחול נכשל: {err}"
    },
    "tray": {
      "normal": "ניטור תקין",
      "warning": "תנודות במכשיר",
      "critical": "התראה"
    },
    "overview": {
      "hint": "ניתוקים זמניים מפעילים התראות והדגשות",
      "exportJson": "ייצוא JSON",
      "exportCsv": "ייצוא CSV",
      "clearHistory": "ניקוי היסטוריה",
      "eventsTitle": "אירועי מכשירים",
      "eventsMeta": "החדשים ביותר קודם",
      "empty": "אין אירועים עדיין. חבר או נתק מכשיר USB/Bluetooth כדי לראות פעילות כאן."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "סריקה",
        "desc": "בדיקת רדיו Bluetooth ושירות bthserv",
        "idle": "לחץ סריקה כדי להתחיל",
        "loading": "סורק…",
        "ok": "תקין",
        "warn": "נמצאו בעיות",
        "unknown": "לא ידוע",
        "radio": "מכשירי רדיו",
        "radioCount": "{n} מכשירים",
        "issues": "בעיות",
        "noIssues": "אין בעיות"
      },
      "bsod": {
        "title": "מעקב BSOD",
        "scan": "סריקה",
        "desc": "ניתוח Minidump ואירועי BugCheck",
        "idle": "לחץ סריקה כדי למצוא קבצי dump",
        "loading": "סורק…",
        "none": "לא נמצאו קבצי Minidump",
        "recent": "BSOD אחרון",
        "history": "רשומה היסטורית",
        "bugcheck": "בדיקת באג",
        "driver": "מנהל התקן",
        "dumpPath": "נתיב dump"
      },
      "repair": {
        "title": "תיקון בלחיצה אחת",
        "run": "הפעל תיקון",
        "desc": "הפעלה מחדש של שירותי Bluetooth ושמע; סריקת השהיה סלקטיבית של USB",
        "idle": "מפעיל מחדש את bthserv ו-Audiosrv וסורק הגדרות חשמל USB",
        "loading": "מריץ…",
        "adminHint": "לא פועל כמנהל מערכת — הפעלה מחדש של שירות עלולה להיכשל",
        "adminBanner": "נדרש מנהל מערכת: לחץ ימני על ZeroTick → הפעל כמנהל מערכת",
        "restarted": "שירותים הופעלו מחדש",
        "noneRestarted": "לא הופעלו שירותים מחדש",
        "failed": "פריטים שנכשלו",
        "usbScan": "סריקת חשמל USB",
        "noUsbWarn": "אין צמתי USB עם חיסכון בחשמל מופעל"
      }
    },
    "ports": {
      "hint": "יציאת פיתוח {port} · שחרור שאריות node / vite",
      "scan": "סריקת יציאות",
      "releaseAll": "שחרור הכל",
      "releaseAllN": "שחרור הכל ({n})",
      "releaseOne": "שחרור",
      "scanning": "סורק…",
      "empty": "לחץ סריקת יציאות כדי לראות שימוש מקומי",
      "noListeners": "אין יציאות האזנה מקומיות",
      "reservedTitle": "טווחי החרגת TCP של Windows",
      "category": {
        "releasable": "ניתן לשחרור",
        "in_use": "בשימוש",
        "inuse": "בשימוש",
        "time_wait": "TIME_WAIT",
        "system_reserved": "שמור למערכת",
        "free": "זמין"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — בדרך כלל משתחרר תוך 1–4 דקות",
        "system_reserved": "יציאה בטווח החרגה דינמי של Windows — שנה יציאת פיתוח",
        "self_app": "בשימוש על ידי ZeroTick — לא ניתן לסיים",
        "protected": "תהליך מערכת/קריטי — לא ניתן לשחרר",
        "releasable": "שארית פיתוח — בטוח לסיים",
        "in_use": "בשימוש על ידי אפליקציה אחרת — סיום עלול לגרום לבעיות",
        "unknown": "תהליך לא ידוע — לא ניתן לסווג כשארית",
        "free": "יציאה זמינה לשרת פיתוח",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "ניטור",
      "groupData": "היסטוריה",
      "locale": "שפת ממשק",
      "threshold": "סף זמני",
      "trayRecovery": "שחזור התראת מגש",
      "bluetoothPoll": "מרווח סקר Bluetooth",
      "historyMax": "שמירת היסטוריה",
      "timelineMax": "תצוגת רשימת אירועים",
      "nativeNotify": "התראות במצב מזעור",
      "launchStartup": "הפעלה בכניסה",
      "save": "שמירה",
      "groupGeneral": "כללי"
    },
    "units": {
      "ms": "מ״ש",
      "sec": "שנ׳",
      "count": "פריטים"
    },
    "events": {
      "transient": "זמני",
      "remove": "מנותק",
      "arrival": "מחובר",
      "unknownDevice": "מכשיר לא ידוע",
      "device": "מכשיר"
    },
    "toast": {
      "saved": "ההגדרות נשמרו",
      "saveFailed": "השמירה נכשלה: {err}",
      "historyCleared": "ההיסטוריה נוקתה",
      "clearFailed": "הניקוי נכשל: {err}",
      "exported": "יוצא אל {path}",
      "transient": "זמני: {name} ({ms}מ״ש)",
      "disconnected": "מנותק: {name}",
      "bluetooth": "בעיית Bluetooth: {msg}",
      "bsod": "התראת BSOD: {code}",
      "repairFailed": "התיקון נכשל: {err}",
      "pidKilled": "הופסק PID {pid}",
      "releasedN": "שוחררו {n} תהליכים",
      "nothingToRelease": "אין מה לשחרר"
    },
    "spin": {
      "increase": "הגדל",
      "decrease": "הקטן"
    },
    "app": {
      "title": "ZeroTick — אבחון מערכת"
    }
  },
  "ms": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Gambaran keseluruhan",
      "diagnostics": "Diagnostik",
      "ports": "Port",
      "settings": "Tetapan"
    },
    "pages": {
      "overview": {
        "title": "Gambaran keseluruhan",
        "desc": "Pemantauan masa nyata pemutusan USB dan Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnostik",
        "desc": "Perkhidmatan Bluetooth, penjejakan BSOD dan pembaikan satu klik"
      },
      "ports": {
        "title": "Port",
        "desc": "Lihat dan urus penggunaan port tempatan"
      },
      "settings": {
        "title": "Tetapan",
        "desc": "Pemantauan, sejarah dan pilihan"
      }
    },
    "status": {
      "init": "Memulakan…",
      "running": "Enjin berjalan",
      "failed": "Permulaan gagal: {err}"
    },
    "tray": {
      "normal": "Pemantauan OK",
      "warning": "Turun naik peranti",
      "critical": "Amaran"
    },
    "overview": {
      "hint": "Pemutusan sementara mencetuskan amaran dan penyerlahan",
      "exportJson": "Eksport JSON",
      "exportCsv": "Eksport CSV",
      "clearHistory": "Kosongkan sejarah",
      "eventsTitle": "Peristiwa peranti",
      "eventsMeta": "Terbaharu dahulu",
      "empty": "Tiada peristiwa lagi. Palam masuk atau cabut peranti USB/Bluetooth untuk melihat aktiviti di sini."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Imbas",
        "desc": "Semak radio Bluetooth dan perkhidmatan bthserv",
        "idle": "Klik Imbas untuk mula",
        "loading": "Mengimbas…",
        "ok": "Sihat",
        "warn": "Masalah ditemui",
        "unknown": "Tidak diketahui",
        "radio": "Peranti radio",
        "radioCount": "{n} peranti",
        "issues": "Masalah",
        "noIssues": "Tiada masalah"
      },
      "bsod": {
        "title": "Penjejakan BSOD",
        "scan": "Imbas",
        "desc": "Analisis Minidump dan peristiwa BugCheck",
        "idle": "Klik Imbas untuk mencari fail dump",
        "loading": "Mengimbas…",
        "none": "Tiada fail Minidump ditemui",
        "recent": "BSOD terkini",
        "history": "Rekod sejarah",
        "bugcheck": "Bug check",
        "driver": "Pemacu",
        "dumpPath": "Laluan dump"
      },
      "repair": {
        "title": "Pembaikan satu klik",
        "run": "Jalankan pembaikan",
        "desc": "Mulakan semula perkhidmatan Bluetooth dan audio; imbas penggantungan terpilih USB",
        "idle": "Memulakan semula bthserv & Audiosrv dan mengimbas tetapan kuasa USB",
        "loading": "Menjalankan…",
        "adminHint": "Tidak berjalan sebagai pentadbir — mulakan semula perkhidmatan mungkin gagal",
        "adminBanner": "Pentadbir diperlukan: klik kanan ZeroTick → Jalankan sebagai pentadbir",
        "restarted": "Perkhidmatan dimulakan semula",
        "noneRestarted": "Tiada perkhidmatan dimulakan semula",
        "failed": "Item gagal",
        "usbScan": "Imbasan kuasa USB",
        "noUsbWarn": "Tiada nod USB dengan penjimatan kuasa diaktifkan"
      }
    },
    "ports": {
      "hint": "Port dev {port} · Lepaskan sisa node / vite",
      "scan": "Imbas port",
      "releaseAll": "Lepaskan semua",
      "releaseAllN": "Lepaskan semua ({n})",
      "releaseOne": "Lepaskan",
      "scanning": "Mengimbas…",
      "empty": "Klik Imbas port untuk melihat penggunaan tempatan",
      "noListeners": "Tiada port mendengar tempatan",
      "reservedTitle": "Julat pengecualian TCP Windows",
      "category": {
        "releasable": "Boleh dilepaskan",
        "in_use": "Sedang digunakan",
        "inuse": "Sedang digunakan",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Ditempah sistem",
        "free": "Tersedia"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — biasanya dilepaskan dalam 1–4 minit",
        "system_reserved": "Port dalam julat pengecualian dinamik Windows — tukar port dev",
        "self_app": "Digunakan oleh ZeroTick — tidak boleh ditamatkan",
        "protected": "Proses sistem/kritikal — tidak boleh dilepaskan",
        "releasable": "Sisa dev — selamat ditamatkan",
        "in_use": "Digunakan aplikasi lain — penamatan boleh menyebabkan masalah",
        "unknown": "Proses tidak diketahui — tidak boleh diklasifikasi sebagai sisa",
        "free": "Port tersedia untuk pelayan pembangunan",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Pemantauan",
      "groupData": "Sejarah",
      "locale": "Bahasa antara muka",
      "threshold": "Ambang sementara",
      "trayRecovery": "Pemulihan amaran dulang",
      "bluetoothPoll": "Selang tinjauan Bluetooth",
      "historyMax": "Pengekalan sejarah",
      "timelineMax": "Paparan senarai peristiwa",
      "nativeNotify": "Pemberitahuan apabila diminimumkan",
      "launchStartup": "Mula semasa log masuk",
      "save": "Simpan",
      "groupGeneral": "Am"
    },
    "units": {
      "ms": "ms",
      "sec": "saat",
      "count": "item"
    },
    "events": {
      "transient": "Sementara",
      "remove": "Terputus",
      "arrival": "Disambung",
      "unknownDevice": "Peranti tidak diketahui",
      "device": "Peranti"
    },
    "toast": {
      "saved": "Tetapan disimpan",
      "saveFailed": "Simpan gagal: {err}",
      "historyCleared": "Sejarah dikosongkan",
      "clearFailed": "Kosongkan gagal: {err}",
      "exported": "Dieksport ke {path}",
      "transient": "Sementara: {name} ({ms}ms)",
      "disconnected": "Terputus: {name}",
      "bluetooth": "Masalah Bluetooth: {msg}",
      "bsod": "Amaran BSOD: {code}",
      "repairFailed": "Pembaikan gagal: {err}",
      "pidKilled": "PID {pid} ditamatkan",
      "releasedN": "Dilepaskan {n} proses",
      "nothingToRelease": "Tiada untuk dilepaskan"
    },
    "spin": {
      "increase": "Tambah",
      "decrease": "Kurang"
    },
    "app": {
      "title": "ZeroTick — Diagnostik sistem"
    }
  },
  "ro": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Prezentare generală",
      "diagnostics": "Diagnostic",
      "ports": "Porturi",
      "settings": "Setări"
    },
    "pages": {
      "overview": {
        "title": "Prezentare generală",
        "desc": "Monitorizare în timp real a deconectărilor USB și Bluetooth"
      },
      "diagnostics": {
        "title": "Diagnostic",
        "desc": "Serviciu Bluetooth, urmărire BSOD și reparare cu un clic"
      },
      "ports": {
        "title": "Porturi",
        "desc": "Vizualizați și gestionați utilizarea porturilor locale"
      },
      "settings": {
        "title": "Setări",
        "desc": "Monitorizare, istoric și preferințe"
      }
    },
    "status": {
      "init": "Se inițializează…",
      "running": "Motorul rulează",
      "failed": "Inițializare eșuată: {err}"
    },
    "tray": {
      "normal": "Monitorizare OK",
      "warning": "Fluctuație dispozitiv",
      "critical": "Alertă"
    },
    "overview": {
      "hint": "Deconectările tranzitorii declanșează alerte și evidențieri",
      "exportJson": "Exportă JSON",
      "exportCsv": "Exportă CSV",
      "clearHistory": "Șterge istoricul",
      "eventsTitle": "Evenimente dispozitiv",
      "eventsMeta": "Cele mai recente primele",
      "empty": "Niciun eveniment încă. Conectați sau deconectați un dispozitiv USB/Bluetooth pentru a vedea activitatea aici."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Scanează",
        "desc": "Verifică radioul Bluetooth și serviciul bthserv",
        "idle": "Faceți clic pe Scanează pentru a începe",
        "loading": "Se scanează…",
        "ok": "Sănătos",
        "warn": "Probleme găsite",
        "unknown": "Necunoscut",
        "radio": "Dispozitive radio",
        "radioCount": "{n} dispozitiv(e)",
        "issues": "Probleme",
        "noIssues": "Fără probleme"
      },
      "bsod": {
        "title": "Urmărire BSOD",
        "scan": "Scanează",
        "desc": "Analizează Minidump și evenimentele BugCheck",
        "idle": "Faceți clic pe Scanează pentru a găsi fișiere dump",
        "loading": "Se scanează…",
        "none": "Nu s-au găsit fișiere Minidump",
        "recent": "BSOD recent",
        "history": "Înregistrare istorică",
        "bugcheck": "Bug check",
        "driver": "Driver",
        "dumpPath": "Cale dump"
      },
      "repair": {
        "title": "Reparare cu un clic",
        "run": "Rulează repararea",
        "desc": "Repornește serviciile Bluetooth și audio; scanează suspendarea selectivă USB",
        "idle": "Repornește bthserv și Audiosrv și scanează setările de alimentare USB",
        "loading": "Se execută…",
        "adminHint": "Nu rulează ca administrator — repornirea serviciului poate eșua",
        "adminBanner": "Administrator necesar: clic dreapta pe ZeroTick → Rulează ca administrator",
        "restarted": "Servicii repornite",
        "noneRestarted": "Niciun serviciu repornit",
        "failed": "Elemente eșuate",
        "usbScan": "Scanare alimentare USB",
        "noUsbWarn": "Niciun nod USB cu economisire energie activată"
      }
    },
    "ports": {
      "hint": "Port dev {port} · Eliberează resturi node / vite",
      "scan": "Scanează porturi",
      "releaseAll": "Eliberează tot",
      "releaseAllN": "Eliberează tot ({n})",
      "releaseOne": "Eliberează",
      "scanning": "Se scanează…",
      "empty": "Faceți clic pe Scanează porturi pentru a vedea utilizarea locală",
      "noListeners": "Niciun port de ascultare local",
      "reservedTitle": "Intervale de excludere TCP Windows",
      "category": {
        "releasable": "Eliberabil",
        "in_use": "În uz",
        "inuse": "În uz",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Rezervat sistem",
        "free": "Disponibil"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — se eliberează de obicei în 1–4 minute",
        "system_reserved": "Port în intervalul de excludere dinamic Windows — schimbați portul dev",
        "self_app": "Folosit de ZeroTick — nu poate fi terminat",
        "protected": "Proces sistem/critic — nu poate fi eliberat",
        "releasable": "Rest dev — sigur de terminat",
        "in_use": "Folosit de altă aplicație — terminarea poate cauza probleme",
        "unknown": "Proces necunoscut — nu poate fi clasificat ca rest",
        "free": "Port disponibil pentru server de dezvoltare",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Monitorizare",
      "groupData": "Istoric",
      "locale": "Limbă interfață",
      "threshold": "Prag tranzitoriu",
      "trayRecovery": "Recuperare alertă tăviță",
      "bluetoothPoll": "Interval sondare Bluetooth",
      "historyMax": "Păstrare istoric",
      "timelineMax": "Afișare listă evenimente",
      "nativeNotify": "Notificări când este minimizat",
      "launchStartup": "Pornește la conectare",
      "save": "Salvează",
      "groupGeneral": "General"
    },
    "units": {
      "ms": "ms",
      "sec": "sec",
      "count": "elemente"
    },
    "events": {
      "transient": "Tranzitoriu",
      "remove": "Deconectat",
      "arrival": "Conectat",
      "unknownDevice": "Dispozitiv necunoscut",
      "device": "Dispozitiv"
    },
    "toast": {
      "saved": "Setări salvate",
      "saveFailed": "Salvare eșuată: {err}",
      "historyCleared": "Istoric șters",
      "clearFailed": "Ștergere eșuată: {err}",
      "exported": "Exportat în {path}",
      "transient": "Tranzitoriu: {name} ({ms}ms)",
      "disconnected": "Deconectat: {name}",
      "bluetooth": "Problemă Bluetooth: {msg}",
      "bsod": "Alertă BSOD: {code}",
      "repairFailed": "Reparare eșuată: {err}",
      "pidKilled": "PID {pid} terminat",
      "releasedN": "Eliberat(e) {n} proces(e)",
      "nothingToRelease": "Nimic de eliberat"
    },
    "spin": {
      "increase": "Mărește",
      "decrease": "Micșorează"
    },
    "app": {
      "title": "ZeroTick — Diagnostic sistem"
    }
  },
  "hu": {
    "meta": {
      "dir": "ltr"
    },
    "nav": {
      "overview": "Áttekintés",
      "diagnostics": "Diagnosztika",
      "ports": "Portok",
      "settings": "Beállítások"
    },
    "pages": {
      "overview": {
        "title": "Áttekintés",
        "desc": "USB és Bluetooth leválasztások valós idejű monitorozása"
      },
      "diagnostics": {
        "title": "Diagnosztika",
        "desc": "Bluetooth szolgáltatás, BSOD nyomkövetés és egykattintásos javítás"
      },
      "ports": {
        "title": "Portok",
        "desc": "Helyi porthasználat megtekintése és kezelése"
      },
      "settings": {
        "title": "Beállítások",
        "desc": "Megfigyelés, előzmények és beállítások"
      }
    },
    "status": {
      "init": "Inicializálás…",
      "running": "Motor fut",
      "failed": "Inicializálás sikertelen: {err}"
    },
    "tray": {
      "normal": "Monitorozás OK",
      "warning": "Eszköz ingadozás",
      "critical": "Riasztás"
    },
    "overview": {
      "hint": "Átmeneti leválasztások riasztásokat és kiemeléseket váltanak ki",
      "exportJson": "JSON exportálás",
      "exportCsv": "CSV exportálás",
      "clearHistory": "Előzmények törlése",
      "eventsTitle": "Eszköz események",
      "eventsMeta": "Legújabb először",
      "empty": "Még nincsenek események. Csatlakoztasson vagy húzzon ki egy USB/Bluetooth eszközt a tevékenység megtekintéséhez."
    },
    "diag": {
      "bluetooth": {
        "title": "Bluetooth",
        "scan": "Vizsgálat",
        "desc": "Bluetooth rádió és bthserv szolgáltatás ellenőrzése",
        "idle": "Kattintson a Vizsgálat gombra a kezdéshez",
        "loading": "Vizsgálat…",
        "ok": "Egészséges",
        "warn": "Problémák találhatók",
        "unknown": "Ismeretlen",
        "radio": "Rádióeszközök",
        "radioCount": "{n} eszköz",
        "issues": "Problémák",
        "noIssues": "Nincs probléma"
      },
      "bsod": {
        "title": "BSOD nyomkövetés",
        "scan": "Vizsgálat",
        "desc": "Minidump és BugCheck események elemzése",
        "idle": "Kattintson a Vizsgálat gombra a dump fájlok kereséséhez",
        "loading": "Vizsgálat…",
        "none": "Nem található Minidump fájl",
        "recent": "Legutóbbi BSOD",
        "history": "Történeti rekord",
        "bugcheck": "Bug check",
        "driver": "Illesztőprogram",
        "dumpPath": "Dump útvonal"
      },
      "repair": {
        "title": "Egykattintásos javítás",
        "run": "Javítás futtatása",
        "desc": "Bluetooth és hangszolgáltatások újraindítása; USB szelektív felfüggesztés vizsgálata",
        "idle": "Újraindítja a bthserv és Audiosrv szolgáltatásokat, és vizsgálja az USB tápellátási beállításokat",
        "loading": "Futtatás…",
        "adminHint": "Nem rendszergazdaként fut — a szolgáltatás újraindítása sikertelen lehet",
        "adminBanner": "Rendszergazda szükséges: jobb klikk a ZeroTicken → Futtatás rendszergazdaként",
        "restarted": "Szolgáltatások újraindítva",
        "noneRestarted": "Nincs újraindított szolgáltatás",
        "failed": "Sikertelen elemek",
        "usbScan": "USB tápellátás vizsgálat",
        "noUsbWarn": "Nincs energiatakarékos USB csomópont"
      }
    },
    "ports": {
      "hint": "Fejlesztői port {port} · node / vite maradványok felszabadítása",
      "scan": "Portok vizsgálata",
      "releaseAll": "Összes felszabadítása",
      "releaseAllN": "Összes felszabadítása ({n})",
      "releaseOne": "Felszabadítás",
      "scanning": "Vizsgálat…",
      "empty": "Kattintson a Portok vizsgálata gombra a helyi használat megtekintéséhez",
      "noListeners": "Nincs helyi figyelő port",
      "reservedTitle": "Windows TCP kizárt tartományok",
      "category": {
        "releasable": "Felszabadítható",
        "in_use": "Használatban",
        "inuse": "Használatban",
        "time_wait": "TIME_WAIT",
        "system_reserved": "Rendszer által fenntartott",
        "free": "Elérhető"
      },
      "message": {
        "time_wait": "TCP TIME_WAIT — általában 1–4 percen belül felszabadul",
        "system_reserved": "Port a Windows dinamikus kizárt tartományában — változtassa a fejlesztői portot",
        "self_app": "A ZeroTick használja — nem lehet leállítani",
        "protected": "Rendszer/kritikus folyamat — nem szabadítható fel",
        "releasable": "Fejlesztői maradvány — biztonságosan leállítható",
        "in_use": "Más alkalmazás használja — a leállítás problémákat okozhat",
        "unknown": "Ismeretlen folyamat — nem osztályozható maradványként",
        "free": "Port elérhető fejlesztői szerverhez",
        "other": "{state}"
      }
    },
    "settings": {
      "groupMonitor": "Megfigyelés",
      "groupData": "Előzmények",
      "locale": "Felület nyelve",
      "threshold": "Átmeneti küszöb",
      "trayRecovery": "Tálca riasztás helyreállítás",
      "bluetoothPoll": "Bluetooth lekérdezési intervallum",
      "historyMax": "Előzmények megőrzése",
      "timelineMax": "Eseménylista megjelenítés",
      "nativeNotify": "Értesítések minimalizáláskor",
      "launchStartup": "Indítás bejelentkezéskor",
      "save": "Mentés",
      "groupGeneral": "Általános"
    },
    "units": {
      "ms": "ms",
      "sec": "mp",
      "count": "elem"
    },
    "events": {
      "transient": "Átmeneti",
      "remove": "Leválasztva",
      "arrival": "Csatlakoztatva",
      "unknownDevice": "Ismeretlen eszköz",
      "device": "Eszköz"
    },
    "toast": {
      "saved": "Beállítások mentve",
      "saveFailed": "Mentés sikertelen: {err}",
      "historyCleared": "Előzmények törölve",
      "clearFailed": "Törlés sikertelen: {err}",
      "exported": "Exportálva ide: {path}",
      "transient": "Átmeneti: {name} ({ms}ms)",
      "disconnected": "Leválasztva: {name}",
      "bluetooth": "Bluetooth probléma: {msg}",
      "bsod": "BSOD riasztás: {code}",
      "repairFailed": "Javítás sikertelen: {err}",
      "pidKilled": "PID {pid} leállítva",
      "releasedN": "{n} folyamat felszabadítva",
      "nothingToRelease": "Nincs mit felszabadítani"
    },
    "spin": {
      "increase": "Növelés",
      "decrease": "Csökkentés"
    },
    "app": {
      "title": "ZeroTick — Rendszerdiagnosztika"
    }
  }
};
