// Home page Logic Layer
Page({
    data: {
        greeting: '',
        visitCount: 0
    },

    onLoad: function() {
        skip.setNavigationBarTitle({ title: 'nav.home' });
        skip.log('Home page loaded');
        try {
            var file = skip.fs.root.getFileHandle('visits.txt');
            var count = parseInt(file.read()) || 0;
            count = count + 1;
            this.setData({ visitCount: count, greeting: skip.i18n.t('home.welcomeBack') });
            var saveFile = skip.fs.root.getFileHandle('visits.txt', { create: true });
            saveFile.write(String(count));
        } catch (e) {
            this.setData({ visitCount: 1, greeting: skip.i18n.t('home.welcome') });
            var saveFile = skip.fs.root.getFileHandle('visits.txt', { create: true });
            saveFile.write('1');
        }
    },

    onShow: function() {
        skip.log('Home page shown');
    }
});
