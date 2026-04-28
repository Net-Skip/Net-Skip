App({
    onLaunch: function(options) {
        skip.log('Weather app launched');
    },
    onShow: function() {
        skip.log('Weather app visible');
    },
    onHide: function() {
        skip.log('Weather app hidden');
    }
});
