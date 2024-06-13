#include <windows.h>

UINT32 ExtractIconImpl(LPCWSTR path, UINT8** output_buf, UINT64* width, UINT64* height, UINT64* bwidth) {
    HICON hicon = NULL;
    UINT ExtractIconReturn = ExtractIconExW(path,0,&hicon,NULL,10);
    if (hicon == NULL) {
        return GetLastError();
    }
    ICONINFO icon_info;
    // extract icon
    if (GetIconInfo(hicon, &icon_info) == 0) {
        DestroyIcon(hicon);
        return GetLastError();
    }
    BITMAP bmp;
    // get bitmap info
    if (GetObject(icon_info.hbmColor, sizeof(bmp), &bmp) == 0) {
        DestroyIcon(hicon);
        return GetLastError();
    }
    // allocate memory
    *output_buf = (UINT8*)malloc(bmp.bmWidthBytes * bmp.bmHeight);
    if (*output_buf == NULL) {
        DestroyIcon(hicon);
        return GetLastError();
    }
    // copy bitmap data
    GetBitmapBits(icon_info.hbmColor, bmp.bmWidthBytes * bmp.bmHeight, *output_buf);
    // set output values
    *width = bmp.bmWidth;
    *height = bmp.bmHeight;
    *bwidth = bmp.bmWidthBytes;
    // free resources
    DeleteObject(icon_info.hbmColor);
    return 0;
}

void FreeMemory(UINT8* buf) {
    if (buf != NULL) {
        free(buf);
    }
}

UINT32 OpenFile(LPCWSTR path) {
    HINSTANCE hInstance = ShellExecuteW(NULL, L"open", path, NULL, NULL, SW_SHOWNORMAL);
    if ((UINT32)hInstance <= 32) {
        return GetLastError();
    }
    return 0;
}

UINT32 GetScreenSize(UINT64* width, UINT64* height) {
    int w = GetSystemMetrics(SM_CXSCREEN);
    int h = GetSystemMetrics(SM_CYSCREEN);
    if (w == 0 || h == 0) {
        return GetLastError();
    }
    *width = w;
    *height = h;
    return 0;
}