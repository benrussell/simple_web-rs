extern crate reqwest;
use reqwest::blocking::Client;
use urlencoding::encode;

use std::ffi::{CStr, CString, c_void};
use std::os::raw::{c_char, c_int};
use std::sync::mpsc;//::{self, Sender, Receiver};
use std::sync::Mutex;
use std::thread;


//this is used inside rust code
struct ResponseData{
    url: String,
    response_code: i32,
    headers: String,
    body: String,
    cb_f: Callback,
    refcon: usize,
}


//this is used to format data to send back to C via callback and event pump
#[repr(C)]
pub struct simple_web_CResponseData {
    response_code: i32,
    url_size: usize,
    header_size: usize,
    body_size: usize,
    url: *const c_char,
    header: *const c_char,
    body: *const c_char,
}


type Callback = extern "C" fn(*const simple_web_CResponseData, refcon: *mut c_void);



// Global static variables
static mut CHANNEL: Option<Mutex<mpsc::Sender<ResponseData>>> = None;
static mut RECEIVER: Option<Mutex<mpsc::Receiver<ResponseData>>> = None;


static mut INIT_REQUIRED: bool = true;



#[no_mangle]
pub extern "C" fn simple_web_init(){

    unsafe{
        if ! INIT_REQUIRED{
            //lib init has already been called - safe eject.
            return;
        }

        // Create channel
        let (tx, rx) = mpsc::channel::<ResponseData>();
        
        // Wrap senders and receivers in mutexes for safe access
        CHANNEL = Some(Mutex::new(tx));
        RECEIVER = Some(Mutex::new(rx));

        INIT_REQUIRED = false;
    }

}






//use url::form_urlencoded::percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};


#[no_mangle]
pub extern "C" fn simple_web_check_gumroad_serial( //url: *const c_char, 
                                            c_product_id: *const c_char, 
                                            c_license_key: *const c_char, 
                                            callback: Callback,
                                            refcon: *mut c_void
                                    ) -> c_int {

    let refcon_ptr_hack = refcon as usize;

    //developers product_id
     // Convert the raw C string to a Rust string
     let c_str = unsafe { CStr::from_ptr(c_product_id) };
     let product_id = match c_str.to_str() {
         Ok(s) => s,
         Err(_) => return -1,
     };

     //users license key
    // Convert the raw C string to a Rust string
    let c_str = unsafe { CStr::from_ptr(c_license_key) };
    let license_key = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let enc_product_id = encode(&product_id);
    let enc_license_key = encode(&license_key);

    let post_body = format!("product_id={}&license_key={}", enc_product_id, enc_license_key);

    let gumroad_query_url = "https://api.gumroad.com/v2/licenses/verify";
    let url_str = gumroad_query_url;


    // Spawn a thread to perform the network request
    let _thread_handle = thread::spawn(move || {


    let client = Client::new();

    //let req_result = reqwest::blocking::post(url_str);

    let req_result = client
                                                    .post(url_str)
                                                    .body(post_body)
                                                    .send();


    let response_data = match req_result{
        Ok(resp)=>{
            let mut headers_string = String::new();
            for (name, value) in resp.headers().iter() {
                headers_string.push_str(&format!("{}: {}\n", name, value.to_str().unwrap_or("")));
            }

            ResponseData{
                url: url_str.to_string(),
                response_code: resp.status().as_u16().into(),
                headers: headers_string,
                body: resp.text().unwrap(),
                cb_f: callback,
                refcon: refcon_ptr_hack,
            }
        },
        Err(_error)=>{
            ResponseData{
                url: url_str.to_string(),
                response_code: 500,
                headers: "lib_failure".to_string(),
                body: "lib_failure_body".to_string(),
                cb_f: callback,
                refcon: refcon_ptr_hack,
            }

        }
    };

    //send our response data back to the main thread
    unsafe{
        let tx = CHANNEL.as_ref().unwrap().lock().unwrap();
        let _ = tx.send(response_data);
    }

});

// print!("rs: get_url(..) - handed off to thread.\n");


0
}




#[no_mangle]
pub extern "C" fn simple_web_get(url: *const c_char, 
                                    callback: Callback,
                                    refcon: *mut c_void
                                ) -> c_int {
                                    
    let refcon_ptr_hack = refcon as usize;

    unsafe{
        if INIT_REQUIRED {
            print!("You must call simple_web_init() before use.\n");
            return -1;
        }
    }

    if url.is_null() {
        return -1;
    }


    // print!("rs: get_url(..)\n");


    // Convert the raw C string to a Rust string
    let c_str = unsafe { CStr::from_ptr(url) };
    let url_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };


    // Spawn a thread to perform the network request
    let _thread_handle = thread::spawn(move || {

        let req_result = reqwest::blocking::get(url_str);

        let response_data = match req_result{
            Ok(resp)=>{
                let mut headers_string = String::new();
                for (name, value) in resp.headers().iter() {
                    headers_string.push_str(&format!("{}: {}\n", name, value.to_str().unwrap_or("")));
                }

                ResponseData{
                    url: url_str.to_string(),
                    response_code: resp.status().as_u16().into(),
                    headers: headers_string,
                    body: resp.text().unwrap(),
                    cb_f: callback,
                    refcon: refcon_ptr_hack,
                }
            },
            Err(_error)=>{
                ResponseData{
                    url: url_str.to_string(),
                    response_code: 500,
                    headers: "lib_failure".to_string(),
                    body: "lib_failure_body".to_string(),
                    cb_f: callback,
                    refcon: refcon_ptr_hack,
                }

            }
        };

        //send our response data back to the main thread
        unsafe{
            let tx = CHANNEL.as_ref().unwrap().lock().unwrap();
            let _ = tx.send(response_data);
        }

    });

    // print!("rs: get_url(..) - handed off to thread.\n");

    
    0
}




#[no_mangle]
pub extern "C" fn simple_web_event_pump(){

    // print!("rs: event pump..\n");

    let msg = unsafe{
        let rx = RECEIVER.as_ref().unwrap().lock().unwrap();

        //get a msg from our thread comms channel
        let msg = match rx.try_recv(){
            Ok(data) => Some(data),
            Err(_msg) => {
                //debugln!("xa-drm: thread_rx.try_recv() failed: {}", _msg);
                None
            },
        };

        msg //ret this from unsafe scope
    };


    if ! msg.is_some(){
        return;
    }

    let response_data = msg.unwrap();

    //we have data to send!
    // print!("rs: event pump wants to send data back to cb\n");
    // print!("rs: {}\n", response_data.url);
    // print!( "{}", response_data.body );


    let callback = response_data.cb_f;

    // print!("rs: exec C callback\n");

    let len_url = response_data.url.len();
    let len_header = response_data.headers.len();
    let len_body = response_data.body.len();
    let c_url = CString::new(response_data.url).unwrap();
    let c_header = CString::new(response_data.headers).unwrap();
    let c_body = CString::new(response_data.body).unwrap();

    let c_resp = simple_web_CResponseData{
        response_code: response_data.response_code,
        url_size: len_url,
        header_size: len_header,
        body_size: len_body,
        url: c_url.as_ptr(),
        header: c_header.as_ptr(),
        body: c_body.as_ptr(),
    };


    let refcon = response_data.refcon as *mut c_void;


    callback( &c_resp, refcon );


}
