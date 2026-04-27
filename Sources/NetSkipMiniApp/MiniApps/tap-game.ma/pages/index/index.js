// Logic Layer — game state and logic runs in JSContext.
// View Layer only displays bound data and sends tap events back.

Page({
    data: {
        score: 0,
        timeLeft: 30,
        bestScore: 0,
        state: 'start',        // 'start', 'playing', 'over'
        targetX: 0,
        targetY: 0,
        targetSize: 50,
        targetVisible: false
    },

    // Internal state (not sent to view)
    _timerId: null,
    _spawnId: null,

    onLoad: function() {
        try {
            var file = skip.fs.root.getFileHandle('best_score.txt');
            var saved = file.read();
            if (saved) {
                this.setData({ bestScore: parseInt(saved, 10) || 0 });
            }
        } catch (e) {
            // No saved score yet
        }
        skip.log('Tap Game loaded, best score: ' + this.data.bestScore);
    },

    onStartGame: function() {
        var self = this;
        self.setData({
            score: 0,
            timeLeft: 30,
            state: 'playing',
            targetVisible: false
        });
        skip.log('Game started');
        self._spawnTarget();

        // Timer countdown
        self._timerId = setInterval(function() {
            var t = self.data.timeLeft - 1;
            self.setData({ timeLeft: t });
            if (t <= 0) {
                self._endGame();
            }
        }, 1000);
    },

    onHitTarget: function() {
        if (this.data.state !== 'playing') return;
        var newScore = this.data.score + 1;
        this.setData({ score: newScore, targetVisible: false });

        // Respawn after brief delay
        var self = this;
        clearTimeout(self._spawnId);
        self._spawnId = setTimeout(function() {
            self._spawnTarget();
        }, 100);
    },

    _spawnTarget: function() {
        if (this.data.state !== 'playing') return;
        // Random position and size (use fixed arena size 300x400 as reference)
        var size = 40 + Math.floor(Math.random() * 30);
        var x = 10 + Math.floor(Math.random() * 250);
        var y = 10 + Math.floor(Math.random() * 350);
        this.setData({
            targetX: x,
            targetY: y,
            targetSize: size,
            targetVisible: true
        });

        // Auto-miss after 1.5s
        var self = this;
        clearTimeout(self._spawnId);
        self._spawnId = setTimeout(function() {
            if (self.data.state === 'playing') {
                self._spawnTarget();
            }
        }, 1500);
    },

    _endGame: function() {
        clearInterval(this._timerId);
        clearTimeout(this._spawnId);

        var best = this.data.bestScore;
        if (this.data.score > best) {
            best = this.data.score;
            skip.fs.root.getFileHandle('best_score.txt', { create: true }).write(String(best));
            skip.log('New best score: ' + best);
        }

        this.setData({
            state: 'over',
            bestScore: best,
            targetVisible: false
        });
        skip.log('Game over. Score: ' + this.data.score + ', Best: ' + best);
    },

    onHide: function() {
        // Clean up timers if page is hidden
        clearInterval(this._timerId);
        clearTimeout(this._spawnId);
    }
});
