@echo off
setlocal

rem Create build directory
if not exist build mkdir build
cd build

rem Configure and build
cmake ..
cmake --build . --config Release

rem Copy the built libraries to the appropriate Unity Plugins directory
cd ..
if not exist ..\Assets\Plugins mkdir ..\Assets\Plugins

rem Copy Windows DLL
copy /Y bin\Windows\MoqNativePlugin.dll ..\Assets\Plugins\

echo Native plugin built and copied to Unity Plugins directory
endlocal 