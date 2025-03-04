using UnityEditor;
using UnityEditor.Build;
using UnityEditor.Build.Reporting;
using System.IO;

public class MoqBuildProcessor : IPreprocessBuildWithReport
{
    public int callbackOrder => 0;

    public void OnPreprocessBuild(BuildReport report)
    {
        // Ensure the native plugins are in the correct location
        string projectRoot = Directory.GetParent(Application.dataPath).FullName;
        string pluginsDir = Path.Combine(Application.dataPath, "Plugins");
        
        // Create plugins directory if it doesn't exist
        if (!Directory.Exists(pluginsDir))
        {
            Directory.CreateDirectory(pluginsDir);
        }
        
        // Copy platform-specific plugins
        #if UNITY_ANDROID
        CopyPluginsForAndroid(projectRoot, pluginsDir);
        #elif UNITY_STANDALONE_WIN
        CopyPluginsForWindows(projectRoot, pluginsDir);
        #elif UNITY_STANDALONE_OSX
        CopyPluginsForMacOS(projectRoot, pluginsDir);
        #elif UNITY_STANDALONE_LINUX
        CopyPluginsForLinux(projectRoot, pluginsDir);
        #endif
        
        AssetDatabase.Refresh();
    }
    
    private void CopyPluginsForAndroid(string projectRoot, string pluginsDir)
    {
        string androidDir = Path.Combine(pluginsDir, "Android");
        if (!Directory.Exists(androidDir))
        {
            Directory.CreateDirectory(androidDir);
        }
        
        // Copy .so files for each architecture
        string[] archs = { "arm64-v8a", "armeabi-v7a", "x86", "x86_64" };
        foreach (string arch in archs)
        {
            string archDir = Path.Combine(androidDir, arch);
            if (!Directory.Exists(archDir))
            {
                Directory.CreateDirectory(archDir);
            }
            
            string source = Path.Combine(projectRoot, "NativePlugins", "Android", arch, "libMoqNativePlugin.so");
            string dest = Path.Combine(archDir, "libMoqNativePlugin.so");
            
            if (File.Exists(source))
            {
                File.Copy(source, dest, true);
            }
            else
            {
                Debug.LogWarning($"Missing Android plugin for {arch}. Expected at {source}");
            }
        }
    }
    
    private void CopyPluginsForWindows(string projectRoot, string pluginsDir)
    {
        string source = Path.Combine(projectRoot, "NativePlugins", "Windows", "MoqNativePlugin.dll");
        string dest = Path.Combine(pluginsDir, "MoqNativePlugin.dll");
        
        if (File.Exists(source))
        {
            File.Copy(source, dest, true);
        }
        else
        {
            Debug.LogWarning($"Missing Windows plugin. Expected at {source}");
        }
    }
    
    private void CopyPluginsForMacOS(string projectRoot, string pluginsDir)
    {
        string source = Path.Combine(projectRoot, "NativePlugins", "macOS", "libMoqNativePlugin.dylib");
        string dest = Path.Combine(pluginsDir, "libMoqNativePlugin.dylib");
        
        if (File.Exists(source))
        {
            File.Copy(source, dest, true);
        }
        else
        {
            Debug.LogWarning($"Missing macOS plugin. Expected at {source}");
        }
    }
    
    private void CopyPluginsForLinux(string projectRoot, string pluginsDir)
    {
        string source = Path.Combine(projectRoot, "NativePlugins", "Linux", "libMoqNativePlugin.so");
        string dest = Path.Combine(pluginsDir, "libMoqNativePlugin.so");
        
        if (File.Exists(source))
        {
            File.Copy(source, dest, true);
        }
        else
        {
            Debug.LogWarning($"Missing Linux plugin. Expected at {source}");
        }
    }
} 