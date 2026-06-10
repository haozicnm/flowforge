@echo off
cd /d C:\Users\haozi\dev\flowforge
start /MIN "" target\release\flowforge.exe
timeout /t 3 /nobreak > nul
cd flutter_app
set PATH=C:\flutter\bin;%PATH%
set PUB_HOSTED_URL=https://pub.flutter-io.cn
set FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
set SERVER_URL=http://127.0.0.1:19529
flutter run -d windows
