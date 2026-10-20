// No real files, paths or account data: test the actual owned open_stream body.
#include <windows.h>
#include <wchar.h>
#include <stdio.h>
#define NT_SUCCESS(status) (((LONG)(status))>=0)
#define STATUS_SUCCESS ((LONG)0)
#define STATUS_BAD_INITIAL_PC ((LONG)0xC000000A)
#define PATH_IS_OPEN(flags) (((flags)&1)!=0)
#define PATH_IS_CLOSED(flags) (((flags)&2)!=0)
#define PATH_IS_READ(flags) (((flags)&4)!=0)
typedef void STREAM;
static const WCHAR *Dll_BoxName=L"MivubiEdu02";
static ULONG rules;
static BOOLEAN boxed;
static LONG identity_status=STATUS_SUCCESS;
static int opens, closes, streams, truncations;
static DWORD disposition;
static bool truncate_ok=true, stream_ok=true, seek_ok=true, open_ok=true, retry_open=false;
static DWORD fake_windows(WCHAR *out,UINT capacity){wcscpy_s(out,capacity,L"C:\\Windows");return 10;}
static WCHAR *File_TranslateDosToNtPath(const WCHAR *name){return _wcsdup(name);}
static void Dll_Free(void *p){free(p);}
static ULONG SbieDll_MatchPath(WCHAR,const WCHAR *){return rules;}
static ULONG SbieDll_GetHandlePath(HANDLE,WCHAR *,BOOLEAN *inside){*inside=boxed;return (ULONG)identity_status;}
static HANDLE fake_create(const WCHAR *,DWORD,DWORD,void *,DWORD how,DWORD,HANDLE){++opens;disposition=how;if(!open_ok||(retry_open&&opens==1)){SetLastError(ERROR_ACCESS_DENIED);return INVALID_HANDLE_VALUE;}return (HANDLE)42;}
static BOOL fake_close(HANDLE){++closes;return TRUE;}
static DWORD fake_pointer(HANDLE,LONG,PLONG,DWORD){return seek_ok?0:INVALID_SET_FILE_POINTER;}
static BOOL fake_truncate(HANDLE){++truncations;return truncate_ok;}
static LONG Stream_Open(STREAM **out,HANDLE){++streams;*out=(void *)43;return stream_ok?STATUS_SUCCESS:(LONG)0xC0000022;}
#undef GetWindowsDirectory
#define GetWindowsDirectory fake_windows
#undef CreateFile
#define CreateFile fake_create
#define CloseHandle fake_close
#define SetFilePointer fake_pointer
#define SetEndOfFile fake_truncate
class IPStoreImpl {public: void *open_stream(bool forWrite);};
#include "stream.inc"
struct MSG_HEADER {ULONG msgid; LONG status;};
static int host_calls;
static MSG_HEADER host_reply={0,0}, denied_reply={0,E_ACCESSDENIED};
static MSG_HEADER *short_reply(LONG status){denied_reply.status=status;return &denied_reply;}
#define SHORT_REPLY(status) short_reply(status)
#define MSGID_PSTORE_GET_TYPE_INFO 1
#define MSGID_PSTORE_GET_SUBTYPE_INFO 2
#define MSGID_PSTORE_READ_ITEM 3
#define MSGID_PSTORE_ENUM_TYPES 4
#define MSGID_PSTORE_ENUM_ITEMS 5
class PipeServer {public:static int ImpersonateCaller(MSG_HEADER **){return 0;}};
class PStoreServer {public:
    static MSG_HEADER *Handler(void *,MSG_HEADER *);
    MSG_HEADER *GetTypeInfo(MSG_HEADER *){++host_calls;return &host_reply;}
    MSG_HEADER *GetSubtypeInfo(MSG_HEADER *){++host_calls;return &host_reply;}
    MSG_HEADER *ReadItem(MSG_HEADER *){++host_calls;return &host_reply;}
    MSG_HEADER *EnumTypes(MSG_HEADER *){++host_calls;return &host_reply;}
    MSG_HEADER *EnumItems(MSG_HEADER *){++host_calls;return &host_reply;}
};
#include "service.inc"
#define CHECK(x) do{if(!(x)){fprintf(stderr,"stream check failed at %d: %s\n",__LINE__,#x);return 1;}}while(0)
static void reset(){opens=closes=streams=truncations=0;rules=0;boxed=FALSE;identity_status=STATUS_SUCCESS;truncate_ok=stream_ok=seek_ok=open_ok=true;retry_open=false;Dll_BoxName=L"MivubiEdu02";}
int main(){
    IPStoreImpl store;
    reset();CHECK(store.open_stream(false)==NULL);CHECK(streams==0&&closes==1);
    reset();CHECK(store.open_stream(true)==NULL);CHECK(truncations==0&&streams==0&&closes==1);CHECK(disposition!=CREATE_ALWAYS&&disposition!=TRUNCATE_EXISTING);
    reset();boxed=TRUE;CHECK(store.open_stream(false)==(void *)43);CHECK(streams==1&&truncations==0&&closes==0);
    reset();boxed=TRUE;CHECK(store.open_stream(true)==(void *)43);CHECK(truncations==1&&closes==0);
    reset();boxed=TRUE;identity_status=STATUS_BAD_INITIAL_PC;CHECK(store.open_stream(false)==(void *)43);
    reset();boxed=TRUE;identity_status=(LONG)0xC0000022;CHECK(store.open_stream(false)==NULL);CHECK(streams==0&&closes==1);
    reset();boxed=TRUE;truncate_ok=false;CHECK(store.open_stream(true)==NULL);CHECK(streams==0&&closes==1);
    reset();boxed=TRUE;seek_ok=false;CHECK(store.open_stream(true)==NULL);CHECK(truncations==0&&streams==0&&closes==1);
    reset();boxed=TRUE;retry_open=true;CHECK(store.open_stream(true)==(void *)43);CHECK(opens==2&&disposition==OPEN_EXISTING&&truncations==1);
    reset();open_ok=false;CHECK(store.open_stream(true)==NULL);CHECK(streams==0&&truncations==0&&closes==0);
    reset();boxed=TRUE;stream_ok=false;CHECK(store.open_stream(false)==NULL);CHECK(closes==1);
    const ULONG blocked_rules[]={1,2,4}; for(ULONG blocked: blocked_rules){reset();rules=blocked;boxed=TRUE;CHECK(store.open_stream(true)==NULL);CHECK(opens==0&&truncations==0);}
    reset();Dll_BoxName=NULL;CHECK(store.open_stream(true)==NULL);CHECK(opens==0);
    PStoreServer service;
    for(ULONG id=1;id<=6;++id){MSG_HEADER request={id,0};MSG_HEADER *reply=service.Handler(&service,&request);CHECK(reply&&reply->status==E_ACCESSDENIED);CHECK(host_calls==0);}
    puts("owned storage stream: synthetic handle origin PASS");return 0;
}
