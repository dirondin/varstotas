@echo off
rem Package tests of a release in Windows Sandbox: see sandbox.ps1.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0sandbox.ps1" %*
if "%~1"=="" pause
