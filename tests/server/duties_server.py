#!/usr/bin/env python3
# Copyright © The Daybrite Project
# SPDX-License-Identifier: MPL-2.0
"""A local server for what the fixture pages cannot stand in for: real HTTP responses.

Fixture mode leaves the loopback alone, so a Net Skip started with NETSKIP_FIXTURES=1 (or
without) reaches this server at http://127.0.0.1:8765/ on the iOS Simulator, and on an Android
emulator after `adb reverse tcp:8765 tcp:8765` and `adb reverse tcp:8766 tcp:8766`.

    python3 tests/server/duties_server.py

  http://127.0.0.1:8765/                 an index of everything below
  /download/report.pdf                   an attachment (Content-Disposition)
  /download/slow.bin                     2 MB sent over about eight seconds, to watch and cancel
  /archive.zip                           a type no page shows, with no Content-Disposition
  /auth                                  HTTP Basic: user "ada", password "lovelace"
  /upload                                a file input that reports the chosen file
  https://127.0.0.1:8766/                the same pages behind a self-signed certificate

The certificate is made with openssl in a temporary directory each time the server starts.
"""
import base64
import http.server
import ssl
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

INDEX = b"""<!doctype html><meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">
<title>Duties server</title><body style="font:16px system-ui;margin:24px">
<h1>Duties server</h1>
<p><a id=pdf href="/download/report.pdf">report.pdf (attachment)</a></p>
<p><a id=slow href="/download/slow.bin">slow.bin (2 MB, slow)</a></p>
<p><a id=zip href="/archive.zip">archive.zip (no Content-Disposition)</a></p>
<p><a id=attr href="/plain.txt" download="renamed.txt">plain.txt (download attribute)</a></p>
<p><a id=auth href="/auth">Sign in (ada / lovelace)</a></p>
<p><a id=upload href="/upload">Upload</a></p>
<p><a id=secure href="https://127.0.0.1:8766/">The self-signed twin</a></p>
<p><a id=mail href="mailto:harbormaster@example.com">Write to the harbormaster</a></p>
<p><a id=intent href="intent://open/#Intent;scheme=dutiesnoapp;package=invalid.duties.noapp;S.browser_fallback_url=http%3A%2F%2F127.0.0.1%3A8765%2Fupload;end">An Android intent with a fallback page</a></p>
</body>"""

UPLOAD = b"""<!doctype html><meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">
<title>Upload</title><body style="font:16px system-ui;margin:24px">
<input type=file id=file><p id=picked>no file</p>
<input type=file id=files multiple accept="image/*"><p id=many>no files</p>
<script>
file.onchange = e => { const f = e.target.files[0]; picked.textContent = f ? f.name + ' ' + f.size : 'no file'; };
files.onchange = e => { many.textContent = [...e.target.files].map(f => f.name).join(', ') || 'no files'; };
</script></body>"""

PDF = (b"%PDF-1.1\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n"
       b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n"
       b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]>>endobj\n"
       b"trailer<</Root 1 0 R>>\n%%EOF\n")


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def send(self, status, mime, body, headers=()):
        self.send_response(status)
        self.send_header("Content-Type", mime)
        self.send_header("Content-Length", str(len(body)))
        for name, value in headers:
            self.send_header(name, value)
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/":
            self.send(200, "text/html", INDEX)
        elif path == "/download/report.pdf":
            self.send(200, "application/pdf", PDF,
                      [("Content-Disposition", 'attachment; filename="report.pdf"')])
        elif path == "/download/slow.bin":
            total, chunk = 2_000_000, 50_000
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.send_header("Content-Length", str(total))
            self.send_header("Content-Disposition", 'attachment; filename="slow.bin"')
            self.end_headers()
            try:
                for _ in range(total // chunk):
                    self.wfile.write(b"\0" * chunk)
                    self.wfile.flush()
                    time.sleep(0.2)
            except (BrokenPipeError, ConnectionResetError):
                pass
        elif path == "/archive.zip":
            self.send(200, "application/zip", b"PK\x05\x06" + b"\0" * 18)
        elif path == "/plain.txt":
            self.send(200, "text/plain", b"Plain text, saved under another name.\n")
        elif path == "/upload":
            self.send(200, "text/html", UPLOAD)
        elif path == "/auth":
            expected = "Basic " + base64.b64encode(b"ada:lovelace").decode()
            if self.headers.get("Authorization") == expected:
                self.send(200, "text/html", b"<title>Signed in</title><p id=who>Signed in as ada</p>")
            else:
                self.send(401, "text/html", b"<title>Unauthorized</title><p id=who>Not signed in</p>",
                          [("WWW-Authenticate", 'Basic realm="Harbor office"')])
        else:
            self.send(404, "text/plain", b"not found\n")

    def log_message(self, fmt, *args):
        sys.stderr.write("%s %s\n" % (self.address_string(), fmt % args))


def main():
    plain = http.server.ThreadingHTTPServer(("127.0.0.1", 8765), Handler)
    secure = http.server.ThreadingHTTPServer(("127.0.0.1", 8766), Handler)
    with tempfile.TemporaryDirectory() as tmp:
        key, cert = Path(tmp, "key.pem"), Path(tmp, "cert.pem")
        subprocess.run(
            ["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "2",
             "-keyout", str(key), "-out", str(cert), "-subj", "/CN=duties.invalid"],
            check=True, capture_output=True)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(str(cert), str(key))
        secure.socket = context.wrap_socket(secure.socket, server_side=True)
        threading.Thread(target=secure.serve_forever, daemon=True).start()
        print("http://127.0.0.1:8765/ and https://127.0.0.1:8766/", flush=True)
        try:
            plain.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
