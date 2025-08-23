#include <windows.h>
#include <stdlib.h>

UINT32 ExtractIconImpl(LPCWSTR path, UINT8** output_buf, UINT64* width, UINT64* height, UINT64* bwidth) {
    HICON hicon = NULL;
    // Request exactly one large icon into hicon
    UINT extracted = ExtractIconExW(path, 0, &hicon, NULL, 1);
    if (extracted == 0 || hicon == NULL) {
        return GetLastError();
    }

    ICONINFO icon_info;
    if (GetIconInfo(hicon, &icon_info) == 0) {
        DestroyIcon(hicon);
        return GetLastError();
    }

    BITMAP bmp;
    if (GetObject(icon_info.hbmColor, sizeof(bmp), &bmp) == 0) {
        if (icon_info.hbmColor) DeleteObject(icon_info.hbmColor);
        if (icon_info.hbmMask) DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon);
        return GetLastError();
    }

    SIZE_T total = (SIZE_T)bmp.bmWidthBytes * (SIZE_T)bmp.bmHeight;
    *output_buf = (UINT8*)malloc(total);
    if (*output_buf == NULL) {
        if (icon_info.hbmColor) DeleteObject(icon_info.hbmColor);
        if (icon_info.hbmMask) DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon);
        return GetLastError();
    }

    // Copy raw bits
    LONG copied = GetBitmapBits(icon_info.hbmColor, (LONG)total, *output_buf);
    if (copied == 0) {
        free(*output_buf);
        *output_buf = NULL;
        if (icon_info.hbmColor) DeleteObject(icon_info.hbmColor);
        if (icon_info.hbmMask) DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon);
        return GetLastError();
    }

    *width = (UINT64)bmp.bmWidth;
    *height = (UINT64)bmp.bmHeight;
    *bwidth = (UINT64)bmp.bmWidthBytes;

    if (icon_info.hbmColor) DeleteObject(icon_info.hbmColor);
    if (icon_info.hbmMask) DeleteObject(icon_info.hbmMask);
    DestroyIcon(hicon);
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