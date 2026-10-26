# Local HTTP transfer progress integration check (requires a built macOS bundle).
import http.server, os, pathlib, subprocess, threading, time
root=pathlib.Path(__file__).resolve().parents[3]
body=(root/'examples/demos/workbench.vrst').read_bytes()
half=len(body)//2
class Server(http.server.ThreadingHTTPServer):
    daemon_threads=True
class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        if self.path.startswith('/known'):
            self.send_header('Content-Length',str(len(body)))
        self.send_header('Connection','close')
        self.end_headers()
        self.wfile.write(body[:half]);self.wfile.flush()
        time.sleep(10)
        try:self.wfile.write(body[half:]);self.wfile.flush()
        except (BrokenPipeError,ConnectionResetError):pass
    def log_message(self,*args):pass
server=Server(('127.0.0.1',0),Handler)
threading.Thread(target=server.serve_forever,daemon=True).start()
try:
    for name,total in [('known',f'Some({len(body)})'),('unknown','None')]:
        script=root/f'target/ui-tests/progress-{name}.txt'
        script.write_text(f'wait 10\naction models web\nwait 10\naction models_url http://127.0.0.1:{server.server_port}/{name}.vrst\nwait 240\nexpect model_transfer=[({half}, {total})]\nquit\n')
        env=os.environ.copy();env.update(HOME=str(root/'target/ui-tests/home'),VARSTOTAS_LANG='en',RUST_LOG='info',VARSTOTAS_SCRIPT=str(script))
        logfile=root/f'target/ui-tests/progress-{name}.log'
        with logfile.open('w') as log:
            result=subprocess.run([str(root/'target/Varstotas.app/Contents/MacOS/varstotas')],cwd=root/'target/ui-tests',env=env,stdout=log,stderr=subprocess.STDOUT,timeout=35)
        text=logfile.read_text()
        assert result.returncode==0 and 'expect ok:' in text and 'expect FAIL' not in text, f'{name}: {logfile}'
        print(name,'progress passed',flush=True)
finally:
    server.shutdown();server.server_close()
