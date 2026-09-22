# roc_desk-http migration

The HTTP workspace implementation has been copied here as the first tool
migration. The canonical implementation is now under `lib/src/http_desk` and
`src-web/src/components/HttpDesk`; the host still retains a compatibility copy
until this crate has its host adapters and standalone Tauri shell.

## Remaining extraction work

1. Replace `crate::` host references in the command adapter with the published
   `roc_desk-common` core interfaces.
2. Add the `lib` Cargo package and `standalone` Tauri shell.
3. Point `roc_desk` at a tagged release and remove its compatibility copy.
