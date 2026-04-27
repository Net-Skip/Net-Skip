// Detail page Logic Layer
Page({
    data: {
        itemId: 0,
        title: '',
        description: ''
    },

    onLoad: function(options) {
        var query = options.query || '';
        var params = {};
        query.split('&').forEach(function(pair) {
            var parts = pair.split('=');
            if (parts.length === 2) params[parts[0]] = parts[1];
        });

        var id = parseInt(params.id) || 1;

        var title = skip.i18n.t('list.item' + id + '.title');
        var desc = skip.i18n.t('detail.desc' + id);

        skip.setNavigationBarTitle({ title: title });
        this.setData({
            itemId: id,
            title: title,
            description: desc
        });
        skip.log('Detail loaded for item ' + id);
    },

    onGoBack: function() {
        skip.log('Navigating back from detail');
        skip.navigateBack({ delta: 1 });
    }
});
