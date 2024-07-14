#include "simple_web.h"
#include <stdio.h>

#include <unistd.h>


size_t req_count=0;


void my_callback(const simple_web_CResponseData* resp, void* refcon) {
    printf("C / my_callback Response:\n");
    printf("  response code: %i\n", resp->response_code);
    printf("  url: %s\n", resp->url);
    printf("  headers:\n%s\n", resp->header);
    
    --req_count;
}



int main() {

    printf("simple_web test.cpp v0.4\n");

    printf("- init lib..\n");
    simple_web_init();

    printf("- req url\n");
    simple_web_get("http://example.com", my_callback, (void*)1);
    ++req_count;

    simple_web_get("http://example.com/404", my_callback, (void*)1);
    ++req_count;


    // Periodically call event_pump to process incoming messages
    while (1) {
        simple_web_event_pump();
        // Add some delay between each call to event_pump
        // For example, sleep for 1 second
        // This loop ensures that the program is non-blocking
        // and periodically processes incoming messages
        //printf("- usleep()\n");
        //usleep(1000);
        sleep(1);


        if( req_count == 0 ){
            break;
        }

    }


    printf("test finished.\n");

    return 0;
}