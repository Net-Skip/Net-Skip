// Logic Layer — runs in JSContext, NOT in the WebView.
// Has access to skip.* APIs (storage, fetch, log, navigation).
// Updates the View Layer via this.setData().

Page({
    data: {
        count: 0,
        noteInput: '',
        noteStatus: '',
        networkResult: '',
        spinning: false,
        sysInfo: ''
    },

    onLoad: function() {
        this.setData({ networkResult: skip.i18n.t('network.tapPrompt') });
        skip.log('Demo page loaded');
        var info = skip.getSystemInfo();
        this.setData({
            sysInfo: 'Platform: ' + info.platform + '\nApp: ' + info.appId + '\nVersion: ' + info.version
        });
    },

    // --- Counter ---
    onIncrement: function() {
        this.setData({ count: this.data.count + 1 });
        skip.log('Counter: ' + (this.data.count));
    },
    onDecrement: function() {
        this.setData({ count: this.data.count - 1 });
        skip.log('Counter: ' + (this.data.count));
    },
    onResetCount: function() {
        this.setData({ count: 0 });
        skip.log('Counter reset');
    },

    // --- Notepad ---
    onNoteInput: function(event) {
        this.setData({ noteInput: event.detail.value });
    },
    onSaveNote: function() {
        var file = skip.fs.root.getFileHandle('note.txt', { create: true });
        file.write(this.data.noteInput);
        this.setData({ noteStatus: skip.i18n.t('notepad.saved') });
        skip.log('Note saved: ' + this.data.noteInput);
    },
    onLoadNote: function() {
        try {
            var file = skip.fs.root.getFileHandle('note.txt');
            var note = file.read();
            this.setData({
                noteInput: note,
                noteStatus: note ? skip.i18n.t('notepad.loaded', { note: note }) : skip.i18n.t('notepad.notFound')
            });
        } catch (e) {
            this.setData({ noteStatus: skip.i18n.t('notepad.notFound') });
        }
    },

    // --- Network ---
    onDoGet: async function() {
        this.setData({ networkResult: skip.i18n.t('network.loading') });
        skip.log('GET https://httpbin.org/get');
        try {
            var response = await skip.fetch('https://httpbin.org/get');
            var data = await response.json();
            this.setData({ networkResult: 'Status: ' + response.status + '\nOrigin: ' + data.origin + '\nURL: ' + data.url });
            skip.log('GET success');
        } catch (e) {
            this.setData({ networkResult: 'Error: ' + e.message });
            skip.log('GET error: ' + e.message);
        }
    },
    onDoPost: async function() {
        var payload = { greeting: 'Hello from MiniApp', timestamp: Date.now() };
        this.setData({ networkResult: skip.i18n.t('network.loading') });
        skip.log('POST https://httpbin.org/post');
        try {
            var response = await skip.fetch('https://httpbin.org/post', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            var data = await response.json();
            this.setData({ networkResult: 'Status: ' + response.status + '\nEchoed: ' + data.data });
            skip.log('POST success');
        } catch (e) {
            this.setData({ networkResult: 'Error: ' + e.message });
        }
    },
    onDoHeaders: async function() {
        this.setData({ networkResult: skip.i18n.t('network.loading') });
        try {
            var response = await skip.fetch('https://httpbin.org/headers', {
                headers: { 'X-MiniApp': 'skip-showcase', 'Accept': 'application/json' }
            });
            var data = await response.json();
            var lines = Object.keys(data.headers).map(function(k) { return k + ': ' + data.headers[k]; });
            this.setData({ networkResult: lines.join('\n') });
        } catch (e) {
            this.setData({ networkResult: 'Error: ' + e.message });
        }
    },
    onDoStatus404: async function() {
        this.setData({ networkResult: skip.i18n.t('network.loading') });
        try {
            var response = await skip.fetch('https://httpbin.org/status/404');
            this.setData({ networkResult: 'ok: ' + response.ok + '\nstatus: ' + response.status });
            skip.log('404 response: ok=' + response.ok);
        } catch (e) {
            this.setData({ networkResult: 'Error: ' + e.message });
        }
    },

    // --- Animation ---
    onToggleAnim: function() {
        var spinning = !this.data.spinning;
        this.setData({ spinning: spinning });
        skip.log('Animation ' + (spinning ? 'started' : 'stopped'));
    }
});
