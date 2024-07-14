#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct simple_web_CResponseData {
  int32_t response_code;
  uintptr_t url_size;
  uintptr_t header_size;
  uintptr_t body_size;
  const char *url;
  const char *header;
  const char *body;
};

using Callback = void(*)(const simple_web_CResponseData*, void *refcon);

extern "C" {

void simple_web_init();

int simple_web_check_gumroad_serial(const char *c_product_id,
                                    const char *c_license_key,
                                    Callback callback,
                                    void *refcon);

int simple_web_get(const char *url, Callback callback, void *refcon);

void simple_web_event_pump();

} // extern "C"
