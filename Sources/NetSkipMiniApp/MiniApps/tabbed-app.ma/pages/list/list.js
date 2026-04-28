// List page Logic Layer
Page({
    data: {
        items: []
    },

    onLoad: function() {
        skip.nav.setNavigationBarTitle({ title: 'nav.list' });
        skip.log('List page loaded');
        this.setData({
            items: [
                { id: 1, title: skip.i18n.t('list.item1.title'), desc: skip.i18n.t('list.item1.desc') },
                { id: 2, title: skip.i18n.t('list.item2.title'), desc: skip.i18n.t('list.item2.desc') },
                { id: 3, title: skip.i18n.t('list.item3.title'), desc: skip.i18n.t('list.item3.desc') },
                { id: 4, title: skip.i18n.t('list.item4.title'), desc: skip.i18n.t('list.item4.desc') },
                { id: 5, title: skip.i18n.t('list.item5.title'), desc: skip.i18n.t('list.item5.desc') }
            ]
        });
    },

    onItemTap: function(event) {
        var itemId = event.detail.id;
        skip.log('Navigating to detail for item ' + itemId);
        skip.nav.navigateTo({ url: 'pages/detail/detail', query: 'id=' + itemId });
    }
});
