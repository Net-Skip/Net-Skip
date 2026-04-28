// Profile page Logic Layer
Page({
    data: {
        username: '',
        navInfo: ''
    },

    onLoad: function() {
        skip.nav.setNavigationBarTitle({ title: 'nav.profile' });
        skip.log('Profile page loaded');
        this.setData({
            username: skip.i18n.t('profile.username')
        });
    },

    onShow: function() {
        this.setData({
            navInfo: skip.i18n.t('profile.navInfo')
        });
    },

    onReLaunch: function() {
        skip.log('reLaunch to home');
        skip.nav.reLaunch({ url: 'pages/home/home' });
    },

    onSwitchToList: function() {
        skip.log('switchTab to list');
        skip.nav.switchTab({ url: 'pages/list/list' });
    }
});
