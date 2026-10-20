// Synthetic stores only. The included functions are extracted from owned cred.c.
#include <windows.h>
#include <wincred.h>
#include <wchar.h>
#include <stdio.h>
#define _FX
#define PST_KEY_CURRENT_USER 0
typedef void IEnumPStoreItems;
static void *Cred_PStore;
static int GUID_SANDBOXIE;
static const WCHAR *Cred_SimpleCred = L"SimpleCred-";
static int host_calls, boxed, malformed, store_available = 1, next_item;
static int hooks;
static int storage_hooks;
typedef void *P_PStoreCreateInstance;
static const char *Pst_PStoreCreateInstanceProcName="SyntheticOnly";
#define Dll_CompartmentMode TRUE
#define GetProcAddress(module,name) ((void *)1)
#define SBIEDLL_HOOK(prefix,name) (++storage_hooks);
#define SBIEDLL_HOOK_CRED(name) (++hooks)
#define SbieApi_QueryConfBool(...) TRUE
#define SbieDll_MatchPath(...) 1
#define PATH_IS_OPEN(flags) ((flags)!=0)
static BOOL Cred_PreparePStore(void) { SetLastError(ERROR_ACCESS_DENIED); return store_available; }
static void *Dll_Alloc(size_t n) { return LocalAlloc(LPTR, n); }
static void Dll_Free(void *p) { LocalFree(p); }
static void Cred_CoTaskMemFree(void *p) { LocalFree(p); }
static WCHAR *Cred_GetName(const WCHAR *domain, const WCHAR *target, ULONG type) {
    (void)domain; (void)target; (void)type; return Dll_Alloc(32);
}
static HRESULT read_item(ULONG *length, UCHAR **data) {
    if (!boxed) return E_FAIL;
    *length = boxed == 2 ? sizeof(ULONG) : 8;
    *data = Dll_Alloc(*length); return S_OK;
}
#define IPStore_ReadItem(store,key,g1,g2,name,len,data,info,flags) read_item(len,data)
static HRESULT enum_items(IEnumPStoreItems **items) {
    next_item = 0; *items = (void *)1; return S_OK;
}
#define IPStore_EnumItems(store,key,g1,g2,flags,items) enum_items(items)
static HRESULT next_name(WCHAR **name, ULONG *fetched) {
    const WCHAR text[] = L"SimpleCred-12345678-marker";
    if (!boxed || next_item++) return S_FALSE;
    *name = Dll_Alloc(sizeof(text)); memcpy(*name, text, sizeof(text));
    *fetched = 1; return S_OK;
}
#define IEnumPStoreItems_Next(items,count,name,fetched) next_name(name,fetched)
static void *Cred_Unserialize1(void *data) { (void)data; return malformed ? NULL : (void *)42; }
static void **Cred_UnserializeN(void **data, ULONG *count) {
    if (malformed) return NULL;
    *count = 0; while (data[*count]) ++*count;
    void **result = Dll_Alloc(sizeof(void *) * *count);
    for (ULONG i=0; i<*count; ++i) result[i]=(void *)42;
    return result;
}
static void *Cred_Serialize(void *data, ULONG *length, BOOL unicode) {
    (void)data; (void)unicode; *length=8; return Dll_Alloc(8);
}
static BOOL host_read(const WCHAR *name, ULONG type, ULONG flags, void **out) {
    (void)name; (void)type; (void)flags; ++host_calls; *out=(void *)99; return TRUE;
}
static BOOL host_enum(void *filter, ULONG flags, ULONG *count, void ***out) {
    (void)filter; (void)flags; ++host_calls; *count=1;
    *out=Dll_Alloc(sizeof(void *)); (*out)[0]=(void *)99; return TRUE;
}
#define __sys_CredReadW host_read
#define __sys_CredReadDomainCredentialsW host_enum
#define __sys_CredReadDomainCredentialsA host_enum
#define __sys_CredEnumerateW host_enum
#define __sys_CredEnumerateA host_enum
#define SbieApi_Log(...) ((void)0)
#include "functions.inc"
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"check failed at %d: %s\n",__LINE__,#x); return 1; } } while(0)
int main(void) {
    void *item=(void *)1, **items=(void *)1;
    ULONG count=99;
    CREDENTIAL_TARGET_INFORMATIONW target={0}; target.DnsDomainName=L"synthetic";
    CHECK(Cred_Init(NULL) && hooks==12);
    CHECK(Pst_Init(NULL) && storage_hooks==1);
    CHECK(!Cred_CredReadW(L"marker",1,0,&item));
    CHECK(host_calls==0 && item==NULL && GetLastError()==ERROR_NOT_FOUND);
    CHECK(!Cred_CredReadDomainCredentialsW(&target,0,&count,&items));
    CHECK(host_calls==0 && count==0 && items==NULL);
    CHECK(!Cred_CredEnumerateW(NULL,0,&count,&items));
    CHECK(host_calls==0 && count==0 && items==NULL);
    CHECK(!Cred_CredReadDomainCredentialsA(NULL,0,&count,&items));
    CHECK(host_calls==0 && count==0 && items==NULL && GetLastError()==ERROR_NOT_SUPPORTED);
    CHECK(!Cred_CredEnumerateA(NULL,0,&count,&items));
    CHECK(host_calls==0 && count==0 && items==NULL && GetLastError()==ERROR_NOT_SUPPORTED);
    boxed=1;
    CHECK(Cred_CredReadW(L"marker",1,0,&item) && item==(void *)42);
    CHECK(Cred_CredReadDomainCredentialsW(&target,0,&count,&items) && count==1);
    LocalFree(items);
    CHECK(Cred_CredEnumerateW(NULL,0,&count,&items) && count==1 && items[0]==(void *)42);
    LocalFree(items);
    malformed=1;
    CHECK(!Cred_CredReadW(L"marker",1,0,&item) && item==NULL);
    CHECK(!Cred_CredReadDomainCredentialsW(&target,0,&count,&items) && count==0 && items==NULL);
    CHECK(!Cred_CredEnumerateW(NULL,0,&count,&items) && count==0 && items==NULL);
    boxed=2; malformed=0;
    CHECK(!Cred_CredReadW(L"marker",1,0,&item) && item==NULL);
    store_available=0; item=(void *)1; items=(void *)1; count=99;
    CHECK(!Cred_CredReadW(L"marker",1,0,&item) && item==NULL);
    CHECK(!Cred_CredReadDomainCredentialsW(&target,0,&count,&items) && count==0 && items==NULL);
    CHECK(!Cred_CredEnumerateW(NULL,0,&count,&items) && count==0 && items==NULL);
    CHECK(host_calls==0);
    puts("owned credential routines: synthetic boxed-only reads PASS"); return 0;
}
