@echo off
rem Package tests of a release on Windows: see run.ps1 (double-click, or run.cmd -Full).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run.ps1" %*
set code=%errorlevel%
if "%~1"=="" pause
exit /b %code%
