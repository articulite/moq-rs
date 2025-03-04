using System;
using System.Runtime.InteropServices;
using UnityEngine;

namespace MoqUnity
{
    public class MoqVideoReceiver : MonoBehaviour
    {
        [SerializeField] private string _serverUrl = "https://localhost:4443";
        [SerializeField] private string _streamPath = "desktop";
        [SerializeField] private Material _targetMaterial;
        [SerializeField] private int _initialWidth = 1920;
        [SerializeField] private int _initialHeight = 1080;
        [SerializeField] private int _targetLatencyMs = 100;

        private IntPtr _moqClient = IntPtr.Zero;
        private Texture2D _texture;
        private byte[] _rawFrameData;
        private int _frameWidth;
        private int _frameHeight;
        private GCHandle _pinnedArray;
        private bool _isInitialized = false;

        #region Native Plugin Imports
        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr MoqCreateClient(string serverUrl, string streamPath, int targetLatencyMs);

        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern void MoqDestroyClient(IntPtr client);

        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool MoqUpdateClient(IntPtr client);

        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool MoqGetFrameInfo(IntPtr client, out int width, out int height);

        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern bool MoqGetFrameData(IntPtr client, IntPtr data, int bufferSize);

        [DllImport("MoqNativePlugin", CallingConvention = CallingConvention.Cdecl)]
        private static extern int MoqGetConnectionStatus(IntPtr client);
        #endregion

        void Start()
        {
            InitializeTexture(_initialWidth, _initialHeight);
            _moqClient = MoqCreateClient(_serverUrl, _streamPath, _targetLatencyMs);
            
            if (_moqClient == IntPtr.Zero)
            {
                Debug.LogError("Failed to create MoQ client!");
                return;
            }
            
            _isInitialized = true;
        }

        void Update()
        {
            if (!_isInitialized) return;

            // Update client and process any events
            if (!MoqUpdateClient(_moqClient))
            {
                Debug.LogWarning("MoQ client update failed");
                return;
            }

            // Check connection status
            int status = MoqGetConnectionStatus(_moqClient);
            if (status < 0)
            {
                Debug.LogWarning($"MoQ connection error: {status}");
                return;
            }
            
            // Check if frame size changed
            int newWidth, newHeight;
            if (MoqGetFrameInfo(_moqClient, out newWidth, out newHeight))
            {
                if (newWidth != _frameWidth || newHeight != _frameHeight)
                {
                    Debug.Log($"Frame size changed to {newWidth}x{newHeight}");
                    ResizeTexture(newWidth, newHeight);
                }
            }

            // Get frame data and update texture
            if (_pinnedArray.IsAllocated && MoqGetFrameData(_moqClient, _pinnedArray.AddrOfPinnedObject(), _rawFrameData.Length))
            {
                _texture.LoadRawTextureData(_rawFrameData);
                _texture.Apply();
            }
        }

        private void InitializeTexture(int width, int height)
        {
            _frameWidth = width;
            _frameHeight = height;
            
            _texture = new Texture2D(width, height, TextureFormat.RGBA32, false);
            _rawFrameData = new byte[width * height * 4]; // RGBA = 4 bytes per pixel
            _pinnedArray = GCHandle.Alloc(_rawFrameData, GCHandleType.Pinned);
            
            if (_targetMaterial != null)
            {
                _targetMaterial.mainTexture = _texture;
            }
            else
            {
                // Try to get the material from the renderer if not explicitly set
                var renderer = GetComponent<Renderer>();
                if (renderer != null)
                {
                    renderer.material.mainTexture = _texture;
                }
            }
        }

        private void ResizeTexture(int width, int height)
        {
            // Clean up old resources
            if (_pinnedArray.IsAllocated)
            {
                _pinnedArray.Free();
            }
            
            _frameWidth = width;
            _frameHeight = height;
            
            // Create new texture and buffer
            _texture = new Texture2D(width, height, TextureFormat.RGBA32, false);
            _rawFrameData = new byte[width * height * 4];
            _pinnedArray = GCHandle.Alloc(_rawFrameData, GCHandleType.Pinned);
            
            // Update material
            if (_targetMaterial != null)
            {
                _targetMaterial.mainTexture = _texture;
            }
            else
            {
                var renderer = GetComponent<Renderer>();
                if (renderer != null)
                {
                    renderer.material.mainTexture = _texture;
                }
            }
        }

        void OnDestroy()
        {
            if (_moqClient != IntPtr.Zero)
            {
                MoqDestroyClient(_moqClient);
                _moqClient = IntPtr.Zero;
            }
            
            if (_pinnedArray.IsAllocated)
            {
                _pinnedArray.Free();
            }
        }
    }
} 