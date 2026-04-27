// App-level Logic Layer — runs first, before any page JS.
App({
    onLaunch: function(options) {
        skip.log('Tabbed App launched');
    },
    onShow: function() {
        skip.log('App visible');
    },
    onHide: function() {
        skip.log('App hidden');
    }
});
