// Reviewed translations for release-critical safety and verification messages.
// Regional variants inherit these strings from their source locale.
export const releaseLocalePatches = {
  "zh-TW": {
    toolkit: { usb: { formatVerificationFailed: "Windows 回報格式化已完成，但 ZeroTick 無法確認要求的檔案系統與磁碟區標籤。請先不要複製檔案，重新掃描磁碟區後再繼續。" } },
    diag: { bsod: { eventOnlyEvidence: "Windows 記錄了當機事件，但沒有可用的小型傾印。下方僅依停止代碼分類，並非完整的傾印分析。" } },
    errors: {
      autostartAdminRequired: "請確認一次系統管理員權限，讓 ZeroTick 設定靜默的系統管理員自動啟動。",
      autostartSetupFailed: "Windows 無法設定靜默的系統管理員自動啟動。設定已保留；請開啟進階模式查看原因後再試一次。",
      bluetoothVerificationFailed: "Windows 已接受 Bluetooth 操作，但裝置狀態未能確認操作完成。請先重新掃描，再決定是否重試。",
    },
  },
  ja: {
    toolkit: { usb: { formatVerificationFailed: "Windows はフォーマットの完了を報告しましたが、ZeroTick は指定されたファイル システムとボリューム ラベルを確認できませんでした。まだファイルをコピーせず、先にボリュームを再スキャンしてください。" } },
    diag: { bsod: { eventOnlyEvidence: "Windows にクラッシュ イベントは記録されていますが、利用可能なミニダンプがありません。以下は停止コードによる限定的な分類であり、完全なダンプ解析ではありません。" } },
    errors: {
      autostartAdminRequired: "ZeroTick が管理者としてサイレント自動起動を設定できるよう、管理者権限を一度だけ確認してください。",
      autostartSetupFailed: "Windows は管理者としてのサイレント自動起動を設定できませんでした。設定は保持されています。詳細モードで原因を確認してから再試行してください。",
      bluetoothVerificationFailed: "Windows は Bluetooth 操作を受け付けましたが、デバイスの状態から完了を確認できませんでした。再試行する前に再スキャンしてください。",
    },
  },
  ko: {
    toolkit: { usb: { formatVerificationFailed: "Windows에서 포맷이 완료되었다고 보고했지만 ZeroTick이 요청한 파일 시스템과 볼륨 레이블을 확인하지 못했습니다. 아직 파일을 복사하지 말고 먼저 볼륨을 다시 검사하세요." } },
    diag: { bsod: { eventOnlyEvidence: "Windows에 충돌 이벤트가 기록되었지만 사용할 수 있는 미니덤프가 없습니다. 아래 분류는 중지 코드만을 기반으로 하며 전체 덤프 분석이 아닙니다." } },
    errors: {
      autostartAdminRequired: "ZeroTick이 관리자 권한으로 조용히 자동 시작되도록 설정하려면 관리자 권한을 한 번만 확인하세요.",
      autostartSetupFailed: "Windows에서 관리자 권한 자동 시작을 설정하지 못했습니다. 설정은 유지되었습니다. 고급 모드에서 원인을 확인한 후 다시 시도하세요.",
      bluetoothVerificationFailed: "Windows에서 Bluetooth 작업을 수락했지만 장치 상태로 완료를 확인하지 못했습니다. 다시 시도하기 전에 먼저 검사하세요.",
    },
  },
  de: {
    toolkit: { usb: { formatVerificationFailed: "Windows meldete den Abschluss der Formatierung, aber ZeroTick konnte das angeforderte Dateisystem und die Volumebezeichnung nicht bestätigen. Kopieren Sie noch keine Dateien, sondern prüfen Sie das Volume zuerst erneut." } },
    diag: { bsod: { eventOnlyEvidence: "Windows hat das Absturzereignis protokolliert, aber es ist kein Minidump verfügbar. Die folgende Einordnung basiert nur auf dem Stoppcode und ist keine vollständige Dumpanalyse." } },
    errors: {
      autostartAdminRequired: "Bestätigen Sie den Administratorzugriff einmal, damit ZeroTick den stillen Autostart mit Administratorrechten einrichten kann.",
      autostartSetupFailed: "Windows konnte den stillen Autostart mit Administratorrechten nicht einrichten. Die Einstellung wurde beibehalten; prüfen Sie die Ursache im erweiterten Modus und versuchen Sie es erneut.",
      bluetoothVerificationFailed: "Windows hat die Bluetooth-Aktion angenommen, der Gerätestatus bestätigt ihren Abschluss jedoch nicht. Prüfen Sie erneut, bevor Sie den Vorgang wiederholen.",
    },
  },
  fr: {
    toolkit: { usb: { formatVerificationFailed: "Windows a indiqué que le formatage était terminé, mais ZeroTick n’a pas pu confirmer le système de fichiers et le nom de volume demandés. Ne copiez pas encore de fichiers ; analysez d’abord de nouveau le volume." } },
    diag: { bsod: { eventOnlyEvidence: "Windows a enregistré l’événement de plantage, mais aucun minidump n’est disponible. La classification ci-dessous se limite au code d’arrêt et ne constitue pas une analyse complète du vidage." } },
    errors: {
      autostartAdminRequired: "Confirmez une fois l’accès administrateur afin que ZeroTick puisse configurer un démarrage automatique silencieux avec élévation.",
      autostartSetupFailed: "Windows n’a pas pu configurer le démarrage automatique silencieux avec élévation. Le réglage a été conservé ; consultez la cause en mode avancé puis réessayez.",
      bluetoothVerificationFailed: "Windows a accepté l’opération Bluetooth, mais l’état de l’appareil n’en confirme pas l’achèvement. Relancez d’abord l’analyse avant de réessayer.",
    },
  },
  es: {
    toolkit: { usb: { formatVerificationFailed: "Windows indicó que el formateo terminó, pero ZeroTick no pudo confirmar el sistema de archivos y la etiqueta de volumen solicitados. No copie archivos todavía; vuelva a analizar el volumen primero." } },
    diag: { bsod: { eventOnlyEvidence: "Windows registró el evento de bloqueo, pero no hay ningún minivolcado disponible. La clasificación siguiente se limita al código de detención y no es un análisis completo del volcado." } },
    errors: {
      autostartAdminRequired: "Confirme una vez el acceso de administrador para que ZeroTick pueda configurar el inicio automático silencioso con privilegios elevados.",
      autostartSetupFailed: "Windows no pudo configurar el inicio automático silencioso con privilegios elevados. La opción se conservó; revise la causa en el modo avanzado y vuelva a intentarlo.",
      bluetoothVerificationFailed: "Windows aceptó la operación de Bluetooth, pero el estado del dispositivo no confirmó que terminara. Vuelva a analizar antes de intentarlo de nuevo.",
    },
  },
  "pt-BR": {
    toolkit: { usb: { formatVerificationFailed: "O Windows informou que a formatação terminou, mas o ZeroTick não conseguiu confirmar o sistema de arquivos e o rótulo do volume solicitados. Não copie arquivos ainda; verifique o volume novamente primeiro." } },
    diag: { bsod: { eventOnlyEvidence: "O Windows registrou o evento de falha, mas não há um minidump disponível. A classificação abaixo se limita ao código de parada e não é uma análise completa do despejo." } },
    errors: {
      autostartAdminRequired: "Confirme o acesso de administrador uma vez para que o ZeroTick possa configurar a inicialização silenciosa com privilégios elevados.",
      autostartSetupFailed: "O Windows não conseguiu configurar a inicialização silenciosa com privilégios elevados. A opção foi mantida; verifique a causa no modo avançado e tente novamente.",
      bluetoothVerificationFailed: "O Windows aceitou a operação de Bluetooth, mas o estado do dispositivo não confirmou a conclusão. Verifique novamente antes de tentar outra vez.",
    },
  },
  ru: {
    toolkit: { usb: { formatVerificationFailed: "Windows сообщил о завершении форматирования, но ZeroTick не смог подтвердить выбранную файловую систему и метку тома. Пока не копируйте файлы — сначала повторно проверьте том." } },
    diag: { bsod: { eventOnlyEvidence: "Windows зарегистрировал событие сбоя, но минидамп недоступен. Классификация ниже основана только на коде остановки и не является полным анализом дампа." } },
    errors: {
      autostartAdminRequired: "Один раз подтвердите права администратора, чтобы ZeroTick мог настроить тихий автозапуск с повышенными правами.",
      autostartSetupFailed: "Windows не удалось настроить тихий автозапуск с повышенными правами. Параметр сохранён; проверьте причину в расширенном режиме и повторите попытку.",
      bluetoothVerificationFailed: "Windows принял операцию Bluetooth, но состояние устройства не подтвердило её завершение. Сначала повторите проверку, затем решите, нужно ли повторять операцию.",
    },
  },
  ar: {
    toolkit: { usb: { formatVerificationFailed: "أبلغ Windows عن اكتمال التهيئة، لكن ZeroTick لم يتمكن من تأكيد نظام الملفات وتسمية وحدة التخزين المطلوبين. لا تنسخ الملفات بعد؛ أعد فحص وحدة التخزين أولاً." } },
    diag: { bsod: { eventOnlyEvidence: "سجّل Windows حدث التعطل، ولكن لا يتوفر تفريغ مصغر. يعتمد التصنيف أدناه على رمز الإيقاف فقط، وليس تحليلاً كاملاً للتفريغ." } },
    errors: {
      autostartAdminRequired: "أكد وصول المسؤول مرة واحدة ليتمكن ZeroTick من إعداد بدء تشغيل صامت بصلاحيات المسؤول.",
      autostartSetupFailed: "تعذر على Windows إعداد بدء التشغيل الصامت بصلاحيات المسؤول. تم الاحتفاظ بالإعداد؛ راجع السبب في الوضع المتقدم ثم أعد المحاولة.",
      bluetoothVerificationFailed: "قبل Windows عملية Bluetooth، لكن حالة الجهاز لم تؤكد اكتمالها. أعد الفحص أولاً قبل المحاولة مجدداً.",
    },
  },
  hi: {
    toolkit: { usb: { formatVerificationFailed: "Windows ने फ़ॉर्मैट पूरा होने की सूचना दी, लेकिन ZeroTick अनुरोधित फ़ाइल सिस्टम और वॉल्यूम लेबल की पुष्टि नहीं कर सका। अभी फ़ाइलें कॉपी न करें; पहले वॉल्यूम को दोबारा स्कैन करें।" } },
    diag: { bsod: { eventOnlyEvidence: "Windows ने क्रैश घटना दर्ज की, लेकिन कोई मिनीडंप उपलब्ध नहीं है। नीचे का वर्गीकरण केवल स्टॉप कोड पर आधारित है और पूर्ण डंप विश्लेषण नहीं है।" } },
    errors: {
      autostartAdminRequired: "ZeroTick के लिए शांत व्यवस्थापक ऑटोस्टार्ट सेट करने हेतु एक बार व्यवस्थापक पहुँच की पुष्टि करें।",
      autostartSetupFailed: "Windows शांत व्यवस्थापक ऑटोस्टार्ट सेट नहीं कर सका। सेटिंग सुरक्षित रखी गई है; उन्नत मोड में कारण देखकर फिर प्रयास करें।",
      bluetoothVerificationFailed: "Windows ने Bluetooth कार्रवाई स्वीकार की, लेकिन डिवाइस की स्थिति ने उसके पूरा होने की पुष्टि नहीं की। दोबारा प्रयास करने से पहले फिर स्कैन करें।",
    },
  },
  it: {
    toolkit: { usb: { formatVerificationFailed: "Windows ha segnalato il completamento della formattazione, ma ZeroTick non ha potuto confermare il file system e l’etichetta di volume richiesti. Non copiare ancora file; esegui prima una nuova scansione del volume." } },
    diag: { bsod: { eventOnlyEvidence: "Windows ha registrato l’evento di arresto anomalo, ma non è disponibile alcun minidump. La classificazione seguente si basa solo sul codice di arresto e non è un’analisi completa del dump." } },
    errors: {
      autostartAdminRequired: "Conferma una volta l’accesso amministratore affinché ZeroTick possa configurare l’avvio automatico invisibile con privilegi elevati.",
      autostartSetupFailed: "Windows non ha potuto configurare l’avvio automatico invisibile con privilegi elevati. L’impostazione è stata mantenuta; controlla la causa in modalità avanzata e riprova.",
      bluetoothVerificationFailed: "Windows ha accettato l’operazione Bluetooth, ma lo stato del dispositivo non ne ha confermato il completamento. Esegui una nuova scansione prima di riprovare.",
    },
  },
  nl: {
    toolkit: { usb: { formatVerificationFailed: "Windows meldde dat het formatteren was voltooid, maar ZeroTick kon het gevraagde bestandssysteem en volumelabel niet bevestigen. Kopieer nog geen bestanden; scan het volume eerst opnieuw." } },
    diag: { bsod: { eventOnlyEvidence: "Windows heeft de crashgebeurtenis vastgelegd, maar er is geen minidump beschikbaar. De classificatie hieronder is alleen op de stopcode gebaseerd en is geen volledige dumpanalyse." } },
    errors: {
      autostartAdminRequired: "Bevestig eenmaal beheerderstoegang zodat ZeroTick stille automatische start met verhoogde rechten kan instellen.",
      autostartSetupFailed: "Windows kon stille automatische start met verhoogde rechten niet instellen. De instelling is behouden; bekijk de oorzaak in de geavanceerde modus en probeer het opnieuw.",
      bluetoothVerificationFailed: "Windows heeft de Bluetooth-bewerking geaccepteerd, maar de apparaatstatus bevestigde niet dat deze is voltooid. Scan opnieuw voordat u het nogmaals probeert.",
    },
  },
  pl: {
    toolkit: { usb: { formatVerificationFailed: "System Windows zgłosił zakończenie formatowania, ale ZeroTick nie mógł potwierdzić żądanego systemu plików i etykiety woluminu. Nie kopiuj jeszcze plików; najpierw ponownie przeskanuj wolumin." } },
    diag: { bsod: { eventOnlyEvidence: "System Windows zarejestrował zdarzenie awarii, ale minizrzut nie jest dostępny. Poniższa klasyfikacja opiera się wyłącznie na kodzie zatrzymania i nie jest pełną analizą zrzutu." } },
    errors: {
      autostartAdminRequired: "Potwierdź raz dostęp administratora, aby ZeroTick mógł skonfigurować cichy autostart z podwyższonymi uprawnieniami.",
      autostartSetupFailed: "System Windows nie mógł skonfigurować cichego autostartu z podwyższonymi uprawnieniami. Ustawienie zachowano; sprawdź przyczynę w trybie zaawansowanym i spróbuj ponownie.",
      bluetoothVerificationFailed: "System Windows przyjął operację Bluetooth, ale stan urządzenia nie potwierdził jej zakończenia. Przed ponowną próbą wykonaj nowe skanowanie.",
    },
  },
  tr: {
    toolkit: { usb: { formatVerificationFailed: "Windows biçimlendirmenin tamamlandığını bildirdi, ancak ZeroTick istenen dosya sistemini ve birim etiketini doğrulayamadı. Henüz dosya kopyalamayın; önce birimi yeniden tarayın." } },
    diag: { bsod: { eventOnlyEvidence: "Windows kilitlenme olayını kaydetti ancak kullanılabilir bir mini döküm yok. Aşağıdaki sınıflandırma yalnızca durdurma koduna dayanır ve tam bir döküm analizi değildir." } },
    errors: {
      autostartAdminRequired: "ZeroTick’in yönetici olarak sessiz otomatik başlatmayı ayarlayabilmesi için yönetici erişimini bir kez onaylayın.",
      autostartSetupFailed: "Windows yönetici olarak sessiz otomatik başlatmayı ayarlayamadı. Ayar korundu; gelişmiş modda nedeni inceleyip yeniden deneyin.",
      bluetoothVerificationFailed: "Windows Bluetooth işlemini kabul etti ancak cihaz durumu işlemin tamamlandığını doğrulamadı. Yeniden denemeden önce tekrar tarayın.",
    },
  },
  vi: {
    toolkit: { usb: { formatVerificationFailed: "Windows báo đã định dạng xong, nhưng ZeroTick không thể xác nhận hệ thống tệp và nhãn ổ đĩa đã yêu cầu. Chưa sao chép tệp; hãy quét lại ổ đĩa trước." } },
    diag: { bsod: { eventOnlyEvidence: "Windows đã ghi nhận sự kiện lỗi nhưng không có tệp kết xuất nhỏ. Phân loại bên dưới chỉ dựa trên mã dừng, không phải là phân tích kết xuất đầy đủ." } },
    errors: {
      autostartAdminRequired: "Hãy xác nhận quyền quản trị một lần để ZeroTick thiết lập tự khởi động im lặng với quyền quản trị.",
      autostartSetupFailed: "Windows không thể thiết lập tự khởi động im lặng với quyền quản trị. Cài đặt vẫn được giữ; hãy xem nguyên nhân trong chế độ nâng cao rồi thử lại.",
      bluetoothVerificationFailed: "Windows đã chấp nhận thao tác Bluetooth nhưng trạng thái thiết bị không xác nhận thao tác đã hoàn tất. Hãy quét lại trước khi thử lại.",
    },
  },
  th: {
    toolkit: { usb: { formatVerificationFailed: "Windows รายงานว่าฟอร์แมตเสร็จแล้ว แต่ ZeroTick ไม่สามารถยืนยันระบบไฟล์และป้ายชื่อไดรฟ์ข้อมูลที่ขอได้ อย่าเพิ่งคัดลอกไฟล์ ให้สแกนไดรฟ์ข้อมูลอีกครั้งก่อน" } },
    diag: { bsod: { eventOnlyEvidence: "Windows บันทึกเหตุการณ์การขัดข้องไว้ แต่ไม่มีมินิดัมป์ให้ใช้ การจัดประเภทด้านล่างอ้างอิงเฉพาะรหัสหยุดและไม่ใช่การวิเคราะห์ดัมป์แบบสมบูรณ์" } },
    errors: {
      autostartAdminRequired: "ยืนยันสิทธิ์ผู้ดูแลระบบหนึ่งครั้งเพื่อให้ ZeroTick ตั้งค่าการเริ่มอัตโนมัติแบบเงียบด้วยสิทธิ์ผู้ดูแลระบบ",
      autostartSetupFailed: "Windows ไม่สามารถตั้งค่าการเริ่มอัตโนมัติแบบเงียบด้วยสิทธิ์ผู้ดูแลระบบได้ การตั้งค่ายังคงอยู่ โปรดดูสาเหตุในโหมดขั้นสูงแล้วลองอีกครั้ง",
      bluetoothVerificationFailed: "Windows ยอมรับการดำเนินการ Bluetooth แล้ว แต่สถานะอุปกรณ์ไม่ยืนยันว่าเสร็จสมบูรณ์ โปรดสแกนอีกครั้งก่อนลองใหม่",
    },
  },
  id: {
    toolkit: { usb: { formatVerificationFailed: "Windows melaporkan bahwa pemformatan selesai, tetapi ZeroTick tidak dapat mengonfirmasi sistem file dan label volume yang diminta. Jangan salin file dahulu; pindai ulang volume terlebih dahulu." } },
    diag: { bsod: { eventOnlyEvidence: "Windows mencatat peristiwa crash, tetapi tidak ada minidump yang tersedia. Klasifikasi di bawah hanya berdasarkan kode berhenti dan bukan analisis dump lengkap." } },
    errors: {
      autostartAdminRequired: "Konfirmasikan akses administrator satu kali agar ZeroTick dapat menyiapkan mulai otomatis senyap dengan hak administrator.",
      autostartSetupFailed: "Windows tidak dapat menyiapkan mulai otomatis senyap dengan hak administrator. Pengaturan dipertahankan; periksa penyebabnya dalam mode Lanjutan lalu coba lagi.",
      bluetoothVerificationFailed: "Windows menerima operasi Bluetooth, tetapi status perangkat tidak mengonfirmasi bahwa operasi selesai. Pindai ulang sebelum mencoba lagi.",
    },
  },
  cs: {
    toolkit: { usb: { formatVerificationFailed: "Systém Windows oznámil dokončení formátování, ale ZeroTick nemohl potvrdit požadovaný systém souborů a název svazku. Zatím nekopírujte soubory; nejprve svazek znovu prohledejte." } },
    diag: { bsod: { eventOnlyEvidence: "Systém Windows zaznamenal událost pádu, ale není k dispozici minidump. Níže uvedená klasifikace vychází pouze z kódu zastavení a není úplnou analýzou výpisu." } },
    errors: {
      autostartAdminRequired: "Jednou potvrďte přístup správce, aby ZeroTick mohl nastavit tiché automatické spuštění s oprávněními správce.",
      autostartSetupFailed: "Systém Windows nemohl nastavit tiché automatické spuštění s oprávněními správce. Nastavení bylo zachováno; zkontrolujte příčinu v rozšířeném režimu a zkuste to znovu.",
      bluetoothVerificationFailed: "Systém Windows přijal operaci Bluetooth, ale stav zařízení nepotvrdil její dokončení. Před dalším pokusem proveďte nové prohledání.",
    },
  },
  da: {
    toolkit: { usb: { formatVerificationFailed: "Windows rapporterede, at formateringen var fuldført, men ZeroTick kunne ikke bekræfte det ønskede filsystem og disknavn. Kopiér ikke filer endnu; scan først disken igen." } },
    diag: { bsod: { eventOnlyEvidence: "Windows registrerede nedbrudshændelsen, men der er ingen minidump tilgængelig. Klassifikationen nedenfor er kun baseret på stopkoden og er ikke en fuld dumpanalyse." } },
    errors: {
      autostartAdminRequired: "Bekræft administratoradgang én gang, så ZeroTick kan konfigurere lydløs automatisk start med administratorrettigheder.",
      autostartSetupFailed: "Windows kunne ikke konfigurere lydløs automatisk start med administratorrettigheder. Indstillingen blev bevaret; se årsagen i avanceret tilstand, og prøv igen.",
      bluetoothVerificationFailed: "Windows accepterede Bluetooth-handlingen, men enhedens status bekræftede ikke, at den blev fuldført. Scan igen, før du prøver på ny.",
    },
  },
  fi: {
    toolkit: { usb: { formatVerificationFailed: "Windows ilmoitti alustuksen valmistuneen, mutta ZeroTick ei voinut vahvistaa pyydettyä tiedostojärjestelmää ja taltion nimeä. Älä kopioi tiedostoja vielä, vaan tarkista taltio ensin uudelleen." } },
    diag: { bsod: { eventOnlyEvidence: "Windows kirjasi kaatumistapahtuman, mutta pienoisvedosta ei ole saatavilla. Alla oleva luokitus perustuu vain pysäytyskoodiin eikä ole täydellinen vedosanalyysi." } },
    errors: {
      autostartAdminRequired: "Vahvista järjestelmänvalvojan käyttöoikeus kerran, jotta ZeroTick voi määrittää hiljaisen automaattisen käynnistyksen järjestelmänvalvojana.",
      autostartSetupFailed: "Windows ei voinut määrittää hiljaista automaattista käynnistystä järjestelmänvalvojana. Asetus säilytettiin; tarkista syy lisäasetustilassa ja yritä uudelleen.",
      bluetoothVerificationFailed: "Windows hyväksyi Bluetooth-toiminnon, mutta laitteen tila ei vahvistanut sen valmistumista. Tarkista uudelleen ennen uutta yritystä.",
    },
  },
  nb: {
    toolkit: { usb: { formatVerificationFailed: "Windows rapporterte at formateringen var fullført, men ZeroTick kunne ikke bekrefte det valgte filsystemet og volumnavnet. Ikke kopier filer ennå; skann volumet på nytt først." } },
    diag: { bsod: { eventOnlyEvidence: "Windows registrerte krasjhendelsen, men ingen minidump er tilgjengelig. Klassifiseringen nedenfor er bare basert på stoppkoden og er ikke en fullstendig dumpanalyse." } },
    errors: {
      autostartAdminRequired: "Bekreft administratortilgang én gang, slik at ZeroTick kan konfigurere stille automatisk oppstart med administratorrettigheter.",
      autostartSetupFailed: "Windows kunne ikke konfigurere stille automatisk oppstart med administratorrettigheter. Innstillingen ble beholdt; kontroller årsaken i avansert modus og prøv igjen.",
      bluetoothVerificationFailed: "Windows godtok Bluetooth-handlingen, men enhetsstatusen bekreftet ikke at den ble fullført. Skann på nytt før du prøver igjen.",
    },
  },
  sv: {
    toolkit: { usb: { formatVerificationFailed: "Windows rapporterade att formateringen var klar, men ZeroTick kunde inte bekräfta det begärda filsystemet och volymnamnet. Kopiera inga filer ännu; skanna volymen igen först." } },
    diag: { bsod: { eventOnlyEvidence: "Windows registrerade kraschen, men ingen minidump är tillgänglig. Klassificeringen nedan bygger endast på stoppkoden och är inte en fullständig dumpanalys." } },
    errors: {
      autostartAdminRequired: "Bekräfta administratörsåtkomst en gång så att ZeroTick kan konfigurera tyst automatisk start med administratörsbehörighet.",
      autostartSetupFailed: "Windows kunde inte konfigurera tyst automatisk start med administratörsbehörighet. Inställningen behölls; kontrollera orsaken i avancerat läge och försök igen.",
      bluetoothVerificationFailed: "Windows accepterade Bluetooth-åtgärden, men enhetsstatusen bekräftade inte att den slutfördes. Skanna igen innan du försöker på nytt.",
    },
  },
  uk: {
    toolkit: { usb: { formatVerificationFailed: "Windows повідомила про завершення форматування, але ZeroTick не зміг підтвердити вибрану файлову систему й мітку тому. Поки не копіюйте файли — спочатку повторно перевірте том." } },
    diag: { bsod: { eventOnlyEvidence: "Windows зареєструвала подію збою, але мінідамп недоступний. Наведена нижче класифікація ґрунтується лише на коді зупинки й не є повним аналізом дампа." } },
    errors: {
      autostartAdminRequired: "Один раз підтвердьте доступ адміністратора, щоб ZeroTick міг налаштувати тихий автозапуск із підвищеними правами.",
      autostartSetupFailed: "Windows не вдалося налаштувати тихий автозапуск із підвищеними правами. Налаштування збережено; перевірте причину в розширеному режимі й повторіть спробу.",
      bluetoothVerificationFailed: "Windows прийняла операцію Bluetooth, але стан пристрою не підтвердив її завершення. Перед повторною спробою виконайте нове сканування.",
    },
  },
  he: {
    toolkit: { usb: { formatVerificationFailed: "Windows דיווח שהאתחול הושלם, אך ZeroTick לא הצליח לאמת את מערכת הקבצים ואת תווית אמצעי האחסון שנבחרו. אל תעתיק קבצים עדיין; סרוק תחילה את אמצעי האחסון מחדש." } },
    diag: { bsod: { eventOnlyEvidence: "Windows תיעד את אירוע הקריסה, אך אין קובץ minidump זמין. הסיווג שלהלן מבוסס על קוד העצירה בלבד ואינו ניתוח מלא של קובץ dump." } },
    errors: {
      autostartAdminRequired: "אשר גישת מנהל פעם אחת כדי ש-ZeroTick יוכל להגדיר הפעלה אוטומטית שקטה עם הרשאות מנהל.",
      autostartSetupFailed: "Windows לא הצליח להגדיר הפעלה אוטומטית שקטה עם הרשאות מנהל. ההגדרה נשמרה; בדוק את הסיבה במצב מתקדם ונסה שוב.",
      bluetoothVerificationFailed: "Windows קיבל את פעולת ה-Bluetooth, אך מצב ההתקן לא אישר שהפעולה הושלמה. סרוק מחדש לפני ניסיון נוסף.",
    },
  },
  ms: {
    toolkit: { usb: { formatVerificationFailed: "Windows melaporkan bahawa pemformatan selesai, tetapi ZeroTick tidak dapat mengesahkan sistem fail dan label volum yang diminta. Jangan salin fail dahulu; imbas semula volum terlebih dahulu." } },
    diag: { bsod: { eventOnlyEvidence: "Windows merekodkan peristiwa ranap, tetapi tiada minidump tersedia. Pengelasan di bawah hanya berdasarkan kod henti dan bukan analisis dump penuh." } },
    errors: {
      autostartAdminRequired: "Sahkan akses pentadbir sekali supaya ZeroTick boleh menyediakan mula automatik senyap dengan hak pentadbir.",
      autostartSetupFailed: "Windows tidak dapat menyediakan mula automatik senyap dengan hak pentadbir. Tetapan dikekalkan; semak punca dalam mod Lanjutan dan cuba lagi.",
      bluetoothVerificationFailed: "Windows menerima operasi Bluetooth, tetapi status peranti tidak mengesahkan bahawa operasi selesai. Imbas semula sebelum mencuba lagi.",
    },
  },
  ro: {
    toolkit: { usb: { formatVerificationFailed: "Windows a raportat că formatarea s-a încheiat, dar ZeroTick nu a putut confirma sistemul de fișiere și eticheta de volum solicitate. Nu copia încă fișiere; scanează din nou volumul mai întâi." } },
    diag: { bsod: { eventOnlyEvidence: "Windows a înregistrat evenimentul de blocare, dar nu este disponibil niciun minidump. Clasificarea de mai jos se bazează doar pe codul de oprire și nu este o analiză completă a dumpului." } },
    errors: {
      autostartAdminRequired: "Confirmă o singură dată accesul de administrator pentru ca ZeroTick să poată configura pornirea automată silențioasă cu privilegii ridicate.",
      autostartSetupFailed: "Windows nu a putut configura pornirea automată silențioasă cu privilegii ridicate. Setarea a fost păstrată; verifică motivul în modul avansat și încearcă din nou.",
      bluetoothVerificationFailed: "Windows a acceptat operația Bluetooth, dar starea dispozitivului nu a confirmat finalizarea. Scanează din nou înainte de a reîncerca.",
    },
  },
  hu: {
    toolkit: { usb: { formatVerificationFailed: "A Windows befejezettnek jelentette a formázást, de a ZeroTick nem tudta megerősíteni a kért fájlrendszert és kötetcímkét. Még ne másoljon fájlokat; előbb vizsgálja meg újra a kötetet." } },
    diag: { bsod: { eventOnlyEvidence: "A Windows rögzítette az összeomlási eseményt, de nem érhető el minidump. Az alábbi besorolás csak a leállítási kódon alapul, és nem teljes memóriakép-elemzés." } },
    errors: {
      autostartAdminRequired: "Egyszer erősítse meg a rendszergazdai hozzáférést, hogy a ZeroTick beállíthassa a csendes, emelt jogosultságú automatikus indítást.",
      autostartSetupFailed: "A Windows nem tudta beállítani a csendes, emelt jogosultságú automatikus indítást. A beállítás megmaradt; ellenőrizze az okot speciális módban, majd próbálja újra.",
      bluetoothVerificationFailed: "A Windows elfogadta a Bluetooth-műveletet, de az eszköz állapota nem erősítette meg a befejezést. Újrapróbálás előtt vizsgálja meg ismét.",
    },
  },
};

export const releaseSupplementLocalePatches = {
  "zh-TW": {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "資料磁碟區和裝置的唯讀虛擬磁碟機都無法移除（{reason}）。請關閉該磁碟機的檔案總管視窗及廠商解鎖或備份軟體後重試；若仍失敗，請從 Windows 工作列的「安全地移除硬體」再次確認。", veto: { deviceComponent: "此裝置的一個元件" } } },
    settings: { runAsAdminHint: "同時啟用登入自動啟動時，Windows 只需確認一次以建立靜默的最高權限工作；之後登入不會再顯示 UAC" },
    toast: { usbVolumeEjected: "已安全清除並卸載 {letter}: 資料磁碟區，現在可以拔除磁碟機；由於 Windows 尚未將整個複合裝置標記為已移除，其唯讀虛擬磁碟機可能仍會顯示。" },
  },
  ja: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "データ ボリュームとデバイスの読み取り専用仮想ドライブのどちらも取り外せませんでした（{reason}）。このドライブを開いているエクスプローラーとメーカーのロック解除・バックアップ ソフトを閉じて再試行してください。解決しない場合は、Windows タスクバーの［ハードウェアを安全に取り外す］でもう一度確認してください。", veto: { deviceComponent: "このデバイスのコンポーネント" } } },
    settings: { runAsAdminHint: "サインイン時の自動起動を有効にすると、Windows はサイレントな管理者タスクの作成時に一度だけ確認します。以後のサインインでは UAC は表示されません" },
    toast: { usbVolumeEjected: "{letter}: のデータ ボリュームは安全にフラッシュされ、マウント解除されました。ドライブを取り外せます。Windows が複合デバイス全体を取り外し済みにしていないため、読み取り専用仮想ドライブは表示されたままの場合があります。" },
  },
  ko: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "데이터 볼륨과 장치의 읽기 전용 가상 드라이브를 모두 제거하지 못했습니다({reason}). 해당 드라이브의 파일 탐색기 창과 제조업체 잠금 해제 또는 백업 소프트웨어를 닫고 다시 시도하세요. 계속 실패하면 Windows 작업 표시줄의 하드웨어 안전하게 제거에서 다시 확인하세요.", veto: { deviceComponent: "이 장치의 구성 요소" } } },
    settings: { runAsAdminHint: "로그인 시 자동 시작을 켜면 Windows에서 조용한 관리자 작업을 만들 때 한 번만 확인하며, 이후 로그인에서는 UAC가 표시되지 않습니다" },
    toast: { usbVolumeEjected: "{letter}: 데이터 볼륨을 안전하게 플러시하고 분리했습니다. 이제 드라이브를 뽑아도 됩니다. Windows가 전체 복합 장치를 제거된 것으로 표시하지 않아 읽기 전용 가상 드라이브가 계속 보일 수 있습니다." },
  },
  de: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Weder das Datenvolume noch das schreibgeschützte virtuelle Laufwerk des Geräts konnten entfernt werden ({reason}). Schließen Sie Explorer-Fenster für das Laufwerk sowie Entsperr- oder Sicherungssoftware des Herstellers und versuchen Sie es erneut. Falls es weiter fehlschlägt, bestätigen Sie den Vorgang nochmals über „Hardware sicher entfernen“ in der Windows-Taskleiste.", veto: { deviceComponent: "eine Komponente dieses Geräts" } } },
    settings: { runAsAdminHint: "Bei aktiviertem Autostart nach der Anmeldung fragt Windows einmalig nach, um eine stille Aufgabe mit erhöhten Rechten zu erstellen; bei späteren Anmeldungen erscheint keine UAC-Abfrage" },
    toast: { usbVolumeEjected: "Das Datenvolume {letter}: wurde sicher geleert und ausgehängt. Sie können das Laufwerk jetzt abziehen; das schreibgeschützte virtuelle Laufwerk kann sichtbar bleiben, da Windows das gesamte Verbundgerät nicht als entfernt markiert hat." },
  },
  fr: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Ni le volume de données ni le lecteur virtuel en lecture seule de l’appareil n’ont pu être retirés ({reason}). Fermez les fenêtres de l’Explorateur ouvertes sur le lecteur ainsi que les logiciels de déverrouillage ou de sauvegarde du fabricant, puis réessayez. Si l’échec persiste, confirmez à nouveau avec « Retirer le périphérique en toute sécurité » dans la barre des tâches Windows.", veto: { deviceComponent: "un composant de cet appareil" } } },
    settings: { runAsAdminHint: "Lorsque le démarrage à la connexion est activé, Windows demande une seule confirmation pour créer une tâche silencieuse avec élévation ; les connexions suivantes n’affichent plus l’UAC" },
    toast: { usbVolumeEjected: "Le volume de données {letter}: a été vidé et démonté en toute sécurité. Vous pouvez maintenant débrancher le lecteur ; son lecteur virtuel en lecture seule peut rester visible, car Windows n’a pas marqué l’ensemble du périphérique composite comme retiré." },
  },
  es: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "No se pudo quitar ni el volumen de datos ni la unidad virtual de solo lectura del dispositivo ({reason}). Cierre las ventanas del Explorador de archivos abiertas en la unidad y cualquier software de desbloqueo o copia de seguridad del fabricante, y vuelva a intentarlo. Si sigue fallando, confírmelo otra vez desde Quitar hardware de forma segura en la barra de tareas de Windows.", veto: { deviceComponent: "un componente de este dispositivo" } } },
    settings: { runAsAdminHint: "Con el inicio al iniciar sesión activado, Windows solicita confirmación una vez para crear una tarea silenciosa con privilegios elevados; los inicios de sesión posteriores no muestran UAC" },
    toast: { usbVolumeEjected: "El volumen de datos {letter}: se vació y desmontó de forma segura. Ya puede desconectar la unidad; su unidad virtual de solo lectura puede seguir visible porque Windows no ha marcado todo el dispositivo compuesto como retirado." },
  },
  "pt-BR": {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Nem o volume de dados nem a unidade virtual somente leitura do dispositivo puderam ser removidos ({reason}). Feche as janelas do Explorador de Arquivos abertas na unidade e qualquer software do fabricante para desbloqueio ou backup e tente novamente. Se ainda falhar, confirme outra vez em Remover Hardware com Segurança na barra de tarefas do Windows.", veto: { deviceComponent: "um componente deste dispositivo" } } },
    settings: { runAsAdminHint: "Com a inicialização ao entrar ativada, o Windows pede confirmação uma vez para criar uma tarefa silenciosa com privilégios elevados; os próximos logons não exibem o UAC" },
    toast: { usbVolumeEjected: "O volume de dados {letter}: foi liberado e desmontado com segurança. Agora você pode desconectar a unidade; a unidade virtual somente leitura pode continuar visível porque o Windows não marcou todo o dispositivo composto como removido." },
  },
  ru: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Не удалось извлечь ни том с данными, ни виртуальный диск устройства, доступный только для чтения ({reason}). Закройте окна Проводника для этого диска и программы производителя для разблокировки или резервного копирования, затем повторите попытку. Если ошибка сохраняется, снова подтвердите извлечение через значок безопасного извлечения в панели задач Windows.", veto: { deviceComponent: "компонент этого устройства" } } },
    settings: { runAsAdminHint: "Если включён автозапуск при входе, Windows один раз запросит подтверждение для создания тихой задачи с повышенными правами; при последующих входах UAC не появляется" },
    toast: { usbVolumeEjected: "Том данных {letter}: безопасно сброшен и отключён. Теперь накопитель можно отсоединить; виртуальный диск только для чтения может остаться видимым, поскольку Windows не пометила всё составное устройство как извлечённое." },
  },
  ar: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "تعذرت إزالة كل من وحدة تخزين البيانات ومحرك الأقراص الافتراضي للقراءة فقط في الجهاز ({reason}). أغلق نوافذ مستكشف الملفات الخاصة بمحرك الأقراص وأي برنامج لفتح القفل أو النسخ الاحتياطي من الشركة المصنعة، ثم أعد المحاولة. إذا استمر الفشل، فأكد مرة أخرى من «إزالة الأجهزة بأمان» في شريط مهام Windows.", veto: { deviceComponent: "أحد مكونات هذا الجهاز" } } },
    settings: { runAsAdminHint: "عند تمكين البدء مع تسجيل الدخول، يطلب Windows التأكيد مرة واحدة لإنشاء مهمة صامتة بصلاحيات مرتفعة؛ ولن يظهر UAC في عمليات تسجيل الدخول اللاحقة" },
    toast: { usbVolumeEjected: "تم تفريغ وحدة تخزين البيانات {letter}: وإلغاء تحميلها بأمان. يمكنك الآن فصل محرك الأقراص؛ وقد يظل محرك الأقراص الافتراضي للقراءة فقط ظاهراً لأن Windows لم يضع علامة إزالة على الجهاز المركب بالكامل." },
  },
  hi: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "न तो डेटा वॉल्यूम और न ही डिवाइस की केवल-पढ़ने योग्य वर्चुअल ड्राइव हटाई जा सकी ({reason})। ड्राइव के लिए खुले फ़ाइल एक्सप्लोरर और निर्माता के अनलॉक या बैकअप सॉफ़्टवेयर को बंद करके फिर प्रयास करें। फिर भी विफल होने पर Windows टास्कबार में हार्डवेयर सुरक्षित रूप से निकालें से दोबारा पुष्टि करें।", veto: { deviceComponent: "इस डिवाइस का एक घटक" } } },
    settings: { runAsAdminHint: "साइन-इन स्टार्टअप चालू होने पर Windows शांत उन्नत कार्य बनाने के लिए केवल एक बार पुष्टि मांगता है; बाद के साइन-इन में UAC नहीं दिखता" },
    toast: { usbVolumeEjected: "{letter}: डेटा वॉल्यूम सुरक्षित रूप से फ्लश और अनमाउंट किया गया। अब ड्राइव निकाल सकते हैं; Windows ने पूरे संयुक्त डिवाइस को हटाया हुआ चिह्नित नहीं किया है, इसलिए इसकी केवल-पढ़ने योग्य वर्चुअल ड्राइव दिख सकती है।" },
  },
  it: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Non è stato possibile rimuovere né il volume dati né l’unità virtuale di sola lettura del dispositivo ({reason}). Chiudi le finestre di Esplora file aperte sull’unità e gli eventuali software del produttore per sblocco o backup, quindi riprova. Se il problema persiste, conferma di nuovo da Rimozione sicura dell’hardware nella barra delle applicazioni di Windows.", veto: { deviceComponent: "un componente di questo dispositivo" } } },
    settings: { runAsAdminHint: "Con l’avvio all’accesso attivo, Windows chiede una sola conferma per creare un’attività invisibile con privilegi elevati; agli accessi successivi non compare UAC" },
    toast: { usbVolumeEjected: "Il volume dati {letter}: è stato scaricato e smontato in modo sicuro. Ora puoi scollegare l’unità; l’unità virtuale di sola lettura potrebbe restare visibile perché Windows non ha contrassegnato come rimosso l’intero dispositivo composito." },
  },
  nl: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Noch het gegevensvolume, noch het alleen-lezen virtuele station van het apparaat kon worden verwijderd ({reason}). Sluit Verkenner-vensters voor het station en ontgrendelings- of back-upsoftware van de fabrikant en probeer opnieuw. Als het nog steeds mislukt, bevestigt u het opnieuw via Hardware veilig verwijderen in de Windows-taakbalk.", veto: { deviceComponent: "een onderdeel van dit apparaat" } } },
    settings: { runAsAdminHint: "Als opstarten bij aanmelden is ingeschakeld, vraagt Windows eenmaal om bevestiging om een stille taak met verhoogde rechten te maken; bij latere aanmeldingen verschijnt UAC niet" },
    toast: { usbVolumeEjected: "Het gegevensvolume {letter}: is veilig geleegd en ontkoppeld. U kunt het station nu loskoppelen; het alleen-lezen virtuele station kan zichtbaar blijven omdat Windows het volledige samengestelde apparaat niet als verwijderd heeft gemarkeerd." },
  },
  pl: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Nie udało się usunąć ani woluminu danych, ani wirtualnego napędu urządzenia tylko do odczytu ({reason}). Zamknij okna Eksploratora plików dla tego dysku oraz oprogramowanie producenta do odblokowywania lub tworzenia kopii zapasowych i spróbuj ponownie. Jeśli błąd nadal występuje, potwierdź ponownie przez Bezpieczne usuwanie sprzętu na pasku zadań Windows.", veto: { deviceComponent: "składnik tego urządzenia" } } },
    settings: { runAsAdminHint: "Gdy autostart przy logowaniu jest włączony, Windows prosi o potwierdzenie tylko raz podczas tworzenia cichego zadania z podwyższonymi uprawnieniami; przy kolejnych logowaniach UAC się nie pojawia" },
    toast: { usbVolumeEjected: "Wolumin danych {letter}: został bezpiecznie opróżniony i odmontowany. Możesz teraz odłączyć dysk; jego wirtualny napęd tylko do odczytu może pozostać widoczny, ponieważ Windows nie oznaczył całego urządzenia złożonego jako usuniętego." },
  },
  tr: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Ne veri birimi ne de cihazın salt okunur sanal sürücüsü kaldırılamadı ({reason}). Sürücüye ait Dosya Gezgini pencerelerini ve üreticinin kilit açma veya yedekleme yazılımını kapatıp yeniden deneyin. Sorun sürerse Windows görev çubuğundaki Donanımı Güvenle Kaldır seçeneğinden tekrar onaylayın.", veto: { deviceComponent: "bu cihazın bir bileşeni" } } },
    settings: { runAsAdminHint: "Oturum açılışında başlatma etkinse Windows sessiz yükseltilmiş görev oluşturmak için yalnızca bir kez onay ister; sonraki oturum açılışlarında UAC gösterilmez" },
    toast: { usbVolumeEjected: "{letter}: veri birimi güvenle temizlendi ve çıkarıldı. Artık sürücüyü ayırabilirsiniz; Windows bileşik cihazın tamamını kaldırılmış olarak işaretlemediği için salt okunur sanal sürücüsü görünmeye devam edebilir." },
  },
  vi: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Không thể tháo cả ổ dữ liệu lẫn ổ đĩa ảo chỉ đọc của thiết bị ({reason}). Hãy đóng các cửa sổ File Explorer của ổ đĩa và phần mềm mở khóa hoặc sao lưu của nhà sản xuất rồi thử lại. Nếu vẫn không được, hãy xác nhận lại từ Tháo phần cứng an toàn trên thanh tác vụ Windows.", veto: { deviceComponent: "một thành phần của thiết bị này" } } },
    settings: { runAsAdminHint: "Khi bật khởi động lúc đăng nhập, Windows chỉ yêu cầu xác nhận một lần để tạo tác vụ im lặng có quyền nâng cao; các lần đăng nhập sau không hiện UAC" },
    toast: { usbVolumeEjected: "Ổ dữ liệu {letter}: đã được ghi hết bộ đệm và tháo an toàn. Bạn có thể rút ổ đĩa; ổ đĩa ảo chỉ đọc vẫn có thể hiển thị vì Windows chưa đánh dấu toàn bộ thiết bị kết hợp là đã tháo." },
  },
  th: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "ไม่สามารถนำทั้งไดรฟ์ข้อมูลและไดรฟ์เสมือนแบบอ่านอย่างเดียวของอุปกรณ์ออกได้ ({reason}) ปิดหน้าต่าง File Explorer ของไดรฟ์และซอฟต์แวร์ปลดล็อกหรือสำรองข้อมูลของผู้ผลิต แล้วลองอีกครั้ง หากยังไม่สำเร็จ ให้ยืนยันอีกครั้งจากนำฮาร์ดแวร์ออกอย่างปลอดภัยในแถบงาน Windows", veto: { deviceComponent: "ส่วนประกอบของอุปกรณ์นี้" } } },
    settings: { runAsAdminHint: "เมื่อเปิดการเริ่มพร้อมการลงชื่อเข้าใช้ Windows จะขอยืนยันเพียงครั้งเดียวเพื่อสร้างงานแบบเงียบที่มีสิทธิ์สูง การลงชื่อเข้าใช้ครั้งต่อไปจะไม่แสดง UAC" },
    toast: { usbVolumeEjected: "ล้างข้อมูลและยกเลิกการต่อเชื่อมไดรฟ์ข้อมูล {letter}: อย่างปลอดภัยแล้ว คุณสามารถถอดไดรฟ์ได้ ไดรฟ์เสมือนแบบอ่านอย่างเดียวอาจยังแสดงอยู่เพราะ Windows ยังไม่ได้ทำเครื่องหมายว่าอุปกรณ์แบบผสมทั้งหมดถูกนำออกแล้ว" },
  },
  id: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Volume data maupun drive virtual hanya-baca perangkat tidak dapat dilepas ({reason}). Tutup jendela File Explorer untuk drive tersebut serta perangkat lunak pembuka kunci atau pencadangan dari produsen, lalu coba lagi. Jika masih gagal, konfirmasikan kembali melalui Lepaskan Perangkat Keras dengan Aman di taskbar Windows.", veto: { deviceComponent: "komponen perangkat ini" } } },
    settings: { runAsAdminHint: "Saat mulai ketika masuk diaktifkan, Windows meminta konfirmasi sekali untuk membuat tugas senyap dengan hak tinggi; proses masuk berikutnya tidak menampilkan UAC" },
    toast: { usbVolumeEjected: "Volume data {letter}: telah dibersihkan dan dilepas dengan aman. Anda kini dapat mencabut drive; drive virtual hanya-baca mungkin tetap terlihat karena Windows belum menandai seluruh perangkat komposit sebagai dilepas." },
  },
  cs: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Nepodařilo se odebrat datový svazek ani virtuální jednotku zařízení jen pro čtení ({reason}). Zavřete okna Průzkumníka souborů pro tuto jednotku a software výrobce pro odemknutí nebo zálohování a zkuste to znovu. Pokud problém trvá, potvrďte odebrání znovu pomocí Bezpečně odebrat hardware na hlavním panelu Windows.", veto: { deviceComponent: "součást tohoto zařízení" } } },
    settings: { runAsAdminHint: "Když je zapnuté spouštění při přihlášení, Windows požádá o potvrzení jednou při vytvoření tiché úlohy se zvýšenými oprávněními; při dalších přihlášeních se UAC nezobrazí" },
    toast: { usbVolumeEjected: "Datový svazek {letter}: byl bezpečně vyprázdněn a odpojen. Nyní můžete jednotku odpojit; virtuální jednotka jen pro čtení může zůstat viditelná, protože Windows neoznačil celé složené zařízení jako odebrané." },
  },
  da: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Hverken datadisken eller enhedens skrivebeskyttede virtuelle drev kunne fjernes ({reason}). Luk Stifinder-vinduer for drevet samt producentens oplåsnings- eller sikkerhedskopieringssoftware, og prøv igen. Hvis det stadig mislykkes, skal du bekræfte igen via Sikker fjernelse af hardware på proceslinjen i Windows.", veto: { deviceComponent: "en komponent i denne enhed" } } },
    settings: { runAsAdminHint: "Når start ved logon er aktiveret, beder Windows én gang om bekræftelse for at oprette en lydløs opgave med administratorrettigheder; senere logon viser ikke UAC" },
    toast: { usbVolumeEjected: "Datadisken {letter}: blev tømt og afmonteret sikkert. Du kan nu frakoble drevet; det skrivebeskyttede virtuelle drev kan forblive synligt, fordi Windows ikke har markeret hele den sammensatte enhed som fjernet." },
  },
  fi: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Tietotaltiota eikä laitteen vain luku -virtuaaliasemaa voitu poistaa ({reason}). Sulje aseman Resurssienhallinta-ikkunat sekä valmistajan lukituksen poisto- tai varmuuskopiointiohjelmisto ja yritä uudelleen. Jos toiminto epäonnistuu edelleen, vahvista se uudelleen Windowsin tehtäväpalkin Poista laite turvallisesti -toiminnolla.", veto: { deviceComponent: "tämän laitteen osa" } } },
    settings: { runAsAdminHint: "Kun käynnistys kirjautumisen yhteydessä on käytössä, Windows pyytää vahvistuksen kerran hiljaisen korotetun tehtävän luomiseksi; myöhemmillä kirjautumiskerroilla UAC ei tule näkyviin" },
    toast: { usbVolumeEjected: "Tietotaltio {letter}: tyhjennettiin ja irrotettiin turvallisesti. Voit nyt irrottaa aseman; vain luku -virtuaaliasema voi jäädä näkyviin, koska Windows ei ole merkinnyt koko yhdistelmälaitetta poistetuksi." },
  },
  nb: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Verken datavolumet eller enhetens skrivebeskyttede virtuelle stasjon kunne fjernes ({reason}). Lukk Filutforsker-vinduer for stasjonen og produsentens programvare for opplåsing eller sikkerhetskopiering, og prøv igjen. Hvis det fortsatt mislykkes, bekrefter du på nytt via Trygg fjerning av maskinvare på oppgavelinjen i Windows.", veto: { deviceComponent: "en komponent i denne enheten" } } },
    settings: { runAsAdminHint: "Når oppstart ved pålogging er aktivert, ber Windows om bekreftelse én gang for å opprette en stille oppgave med forhøyede rettigheter; senere pålogginger viser ikke UAC" },
    toast: { usbVolumeEjected: "Datavolumet {letter}: ble tømt og avmontert på en sikker måte. Du kan nå koble fra stasjonen; den skrivebeskyttede virtuelle stasjonen kan fortsatt være synlig fordi Windows ikke har merket hele den sammensatte enheten som fjernet." },
  },
  sv: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Varken datavolymen eller enhetens skrivskyddade virtuella enhet kunde tas bort ({reason}). Stäng Utforskaren-fönster för enheten samt tillverkarens upplåsnings- eller säkerhetskopieringsprogram och försök igen. Om det fortfarande misslyckas bekräftar du igen via Säker borttagning av maskinvara i Windows aktivitetsfält.", veto: { deviceComponent: "en komponent i den här enheten" } } },
    settings: { runAsAdminHint: "När start vid inloggning är aktiverad ber Windows om bekräftelse en gång för att skapa en tyst uppgift med förhöjd behörighet; senare inloggningar visar inte UAC" },
    toast: { usbVolumeEjected: "Datavolymen {letter}: tömdes och avmonterades säkert. Du kan nu koppla från enheten; den skrivskyddade virtuella enheten kan fortfarande visas eftersom Windows inte har markerat hela den sammansatta enheten som borttagen." },
  },
  uk: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Не вдалося вилучити ні том даних, ні віртуальний диск пристрою лише для читання ({reason}). Закрийте вікна Провідника для цього диска та програми виробника для розблокування чи резервного копіювання й повторіть спробу. Якщо помилка не зникне, підтвердьте вилучення ще раз через «Безпечне вилучення пристроїв» на панелі завдань Windows.", veto: { deviceComponent: "компонент цього пристрою" } } },
    settings: { runAsAdminHint: "Якщо ввімкнено запуск під час входу, Windows один раз запитає підтвердження для створення тихого завдання з підвищеними правами; під час наступних входів UAC не з’являється" },
    toast: { usbVolumeEjected: "Том даних {letter}: безпечно очищено й відключено. Тепер накопичувач можна від’єднати; віртуальний диск лише для читання може залишатися видимим, оскільки Windows не позначила весь складений пристрій як вилучений." },
  },
  he: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "לא ניתן להסיר את אמצעי אחסון הנתונים או את הכונן הווירטואלי לקריאה בלבד של ההתקן ({reason}). סגור חלונות של סייר הקבצים עבור הכונן ותוכנות פתיחה או גיבוי של היצרן ונסה שוב. אם הפעולה עדיין נכשלת, אשר שוב דרך הסרה בטוחה של חומרה בשורת המשימות של Windows.", veto: { deviceComponent: "רכיב של התקן זה" } } },
    settings: { runAsAdminHint: "כאשר הפעלה בעת כניסה מופעלת, Windows מבקש אישור פעם אחת ליצירת משימה שקטה עם הרשאות מוגברות; בכניסות הבאות לא יוצג UAC" },
    toast: { usbVolumeEjected: "אמצעי אחסון הנתונים {letter}: נוקה ונותק בבטחה. כעת ניתן לנתק את הכונן; הכונן הווירטואלי לקריאה בלבד עשוי להישאר גלוי מפני ש-Windows לא סימן את ההתקן המשולב כולו כהוסר." },
  },
  ms: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Volum data dan pemacu maya baca sahaja peranti tidak dapat dikeluarkan ({reason}). Tutup tetingkap Penjelajah Fail untuk pemacu serta perisian buka kunci atau sandaran pengeluar, kemudian cuba lagi. Jika masih gagal, sahkan sekali lagi melalui Alih Keluar Perkakasan dengan Selamat pada bar tugas Windows.", veto: { deviceComponent: "komponen peranti ini" } } },
    settings: { runAsAdminHint: "Apabila mula semasa log masuk didayakan, Windows meminta pengesahan sekali untuk mencipta tugas senyap dengan hak tinggi; log masuk seterusnya tidak memaparkan UAC" },
    toast: { usbVolumeEjected: "Volum data {letter}: telah dikosongkan dan dinyahlekap dengan selamat. Anda kini boleh mencabut pemacu; pemacu maya baca sahaja mungkin kekal kelihatan kerana Windows belum menandakan keseluruhan peranti komposit sebagai telah dikeluarkan." },
  },
  ro: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Nu s-a putut elimina nici volumul de date, nici unitatea virtuală doar în citire a dispozitivului ({reason}). Închide ferestrele Explorer pentru unitate și orice software de deblocare sau backup al producătorului, apoi încearcă din nou. Dacă operația eșuează în continuare, confirmă din nou prin Eliminarea în siguranță a hardware-ului din bara de activități Windows.", veto: { deviceComponent: "o componentă a acestui dispozitiv" } } },
    settings: { runAsAdminHint: "Când pornirea la conectare este activată, Windows solicită confirmarea o singură dată pentru a crea o activitate silențioasă cu privilegii ridicate; conectările ulterioare nu afișează UAC" },
    toast: { usbVolumeEjected: "Volumul de date {letter}: a fost golit și demontat în siguranță. Acum poți deconecta unitatea; unitatea virtuală doar în citire poate rămâne vizibilă deoarece Windows nu a marcat întregul dispozitiv compozit ca eliminat." },
  },
  hu: {
    toolkit: { usb: { ejectVirtualOpticalBlocked: "Sem az adatkötetet, sem az eszköz írásvédett virtuális meghajtóját nem sikerült eltávolítani ({reason}). Zárja be a meghajtóhoz tartozó Fájlkezelő-ablakokat és a gyártó feloldó- vagy biztonsági mentési programját, majd próbálja újra. Ha továbbra is sikertelen, erősítse meg ismét a Windows tálcáján a Hardver biztonságos eltávolítása lehetőséggel.", veto: { deviceComponent: "az eszköz egyik összetevője" } } },
    settings: { runAsAdminHint: "Ha engedélyezve van a bejelentkezéskori indítás, a Windows egyszer kér megerősítést egy csendes, emelt jogosultságú feladat létrehozásához; a későbbi bejelentkezésekkor nem jelenik meg UAC" },
    toast: { usbVolumeEjected: "A(z) {letter}: adatkötet biztonságosan kiürült és leválasztásra került. A meghajtó most kihúzható; az írásvédett virtuális meghajtó továbbra is látható lehet, mert a Windows nem jelölte eltávolítottnak a teljes összetett eszközt." },
  },
};
