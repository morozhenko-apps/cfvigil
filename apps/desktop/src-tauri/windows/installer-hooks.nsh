; CFVigil: honor Tauri's existing "Delete application data" checkbox.
; This file is included by the native NSIS template. Do not fork installer.nsi.
; The user.service target is keyring 4's default Windows credential name.
; KEYRING_USER = cloudflare-api-token-v1
; KEYRING_SERVICE = com.morozhenko.cfvigil.cloudflare
!macro NSIS_HOOK_POSTUNINSTALL
  ; Tauri has already checked for a running application and removed its
  ; app-data directories when this checkbox was selected.
  ; /UPDATE must NEVER delete credentials during an upgrade.
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    ; CRED_TYPE_GENERIC = 1. Operates only on the current user's vault.
    ; Delete only CFVigil's own target; do not enumerate or clear all keys.
    System::Call 'advapi32::CredDeleteW(w "cloudflare-api-token-v1.com.morozhenko.cfvigil.cloudflare", i 1, i 0) i .r0'
    ${If} $0 = 0
      System::Call 'kernel32::GetLastError() i .r1'
      ; ERROR_NOT_FOUND = 1168: no token remains, which is already clean.
      ${If} $1 <> 1168
        MessageBox MB_ICONEXCLAMATION|MB_OK "CFVigil could not remove its saved Cloudflare token from Windows Credential Manager. Remove the CFVigil credential manually before reinstalling."
        SetErrorLevel 1
      ${EndIf}
    ${EndIf}
  ${EndIf}
!macroend
