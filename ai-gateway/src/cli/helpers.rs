use std::net::SocketAddr;

pub fn show_welcome_banner(addr: &SocketAddr) {
    let banner = format!(
        "{}{}{}",
        "\x1b[36m", // Cyan color start
        r"
               -*******-               
          :-***--------****=           
      --***-----------------***--      
   ****-------------------------****  
   ******--------------------*****=*             _    ___       ____    _  _____ _______        ___ __   __
   *-----****-------------*****-   *            / \  |_ _|     / ___|  / \|_   _| ____\ \      / / \\ \ / /
   *---------*****---*****--**    **           / _ \  | |_____| |  _  / _ \ | | |  _|  \ \ /\ / / _ \\ V / 
   *--------**=   -**------*=    *=*          / ___ \ | |_____| |_| |/ ___ \| | | |___  \ V  V / ___ \| |  
   *-----**-     =*------**=   -*  *         /_/   \_\___|     \____/_/   \_\_| |_____|  \_/\_/_/   \_\_|  
   *---**      **  *----**     *   *  
   ***=     -*=    *---**    *     *                            By Helicone.ai
   *      *        *--*     *-     * 
   *   **          ***     *-      *                             
   ***=            **     *      -**  
      =**--        *:   *  --**=    
          :***-    *   *****          
              --*******--",
        "\x1b[0m" // Reset color
    );

    let welcome_message = "\x1b[1m🚀 Welcome to AI Gateway! \x1b[0m\n\nTry it \
                           out with this example request:";

    let curl_example = format!(
        "\x1b[0mcurl --request POST \\
  --url http://{addr:?}/ai/chat/completions \
         \\
  --header 'Content-Type: application/json' \\
  --data '{{
    \"model\": \"openai/gpt-4o-mini\",
    \"messages\": [
      {{
        \"role\": \"user\",
        \"content\": \"hello world\"
      }}
    ]
  }}'\x1b[0m"
    );

    println!("{banner}\n\n{welcome_message}\n\n{curl_example}\n");
}
