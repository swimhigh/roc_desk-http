@echo off
setlocal
cd /d "%~dp0"
echo Building roc_desk-http (Release)...
cargo build --release -p roc_desk_http_standalone
if errorlevel 1 (
  echo BUILD FAILED: roc_desk-http
  exit /b 1
)
if not exist bin mkdir bin
copy /Y "target\release\roc_desk_http_standalone.exe" "bin\roc_desk-http.exe" >nul
if errorlevel 1 (
  echo COPY FAILED: roc_desk-http
  exit /b 1
)
echo BUILD OK: bin\roc_desk-http.exe
exit /b 0
