// Reminders — SQLite-backed TODO app
// Uses skip.db.exec() and skip.db.query() for persistence.

function formatDate(iso) {
    if (!iso) return '';
    var d = new Date(iso);
    var now = new Date();
    var diff = now.getTime() - d.getTime();
    if (diff < 86400000 && d.getDate() === now.getDate()) {
        var h = d.getHours(); var m = d.getMinutes();
        return (h < 10 ? '0' : '') + h + ':' + (m < 10 ? '0' : '') + m;
    }
    var months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    return months[d.getMonth()] + ' ' + d.getDate();
}

Page({
    data: {
        items: [],
        filter: 'active',
        searchText: '',
        newTitle: '',
        countLabel: '',
        editing: false,
        editId: 0,
        editTitle: '',
        editNote: ''
    },

    onLoad: function() {
        skip.log('Reminders page loaded');
        skip.db.exec("CREATE TABLE IF NOT EXISTS todos (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, note TEXT DEFAULT '', done INTEGER DEFAULT 0, created_at TEXT)", []);
        this.refresh();
    },

    onFilter: function(event) {
        this.setData({ filter: event.detail.mode });
        this.refresh();
    },

    onSearch: function(event) {
        var text = (event && event.detail && event.detail.text) || '';
        this.setData({ searchText: text });
        this.refresh();
    },

    onAdd: function(event) {
        var title = ((event && event.detail && event.detail.title) || this.data.newTitle || '').trim();
        if (!title) return;
        skip.db.exec('INSERT INTO todos (title, created_at) VALUES (?, datetime(?))', [title, 'now']);
        skip.log('Added: ' + title);
        this.setData({ newTitle: '' });
        this.refresh();
    },

    onToggle: function(event) {
        var id = event.detail.id;
        skip.db.exec('UPDATE todos SET done = CASE WHEN done = 0 THEN 1 ELSE 0 END WHERE id = ?', [id]);
        skip.log('Toggled: ' + id);
        this.refresh();
    },

    onDelete: function(event) {
        var id = event.detail.id;
        skip.db.exec('DELETE FROM todos WHERE id = ?', [id]);
        skip.log('Deleted: ' + id);
        this.refresh();
    },

    onEdit: function(event) {
        var id = event.detail.id;
        var rows = skip.db.query('SELECT title, note FROM todos WHERE id = ?', [id]);
        if (rows.length > 0) {
            this.setData({
                editing: true,
                editId: id,
                editTitle: rows[0].title || '',
                editNote: rows[0].note || ''
            });
        }
    },

    onCancelEdit: function() {
        this.setData({ editing: false, editId: 0, editTitle: '', editNote: '' });
    },

    onSaveEdit: function(event) {
        var title = ((event && event.detail && event.detail.title) || this.data.editTitle || '').trim();
        var note = (event && event.detail && event.detail.note) || this.data.editNote || '';
        if (!title) return;
        skip.db.exec('UPDATE todos SET title = ?, note = ? WHERE id = ?', [title, note, this.data.editId]);
        skip.log('Updated: ' + this.data.editId);
        this.setData({ editing: false, editId: 0, editTitle: '', editNote: '' });
        this.refresh();
    },

    refresh: function() {
        var filter = this.data.filter;
        var search = (this.data.searchText || '').trim();
        var sql = 'SELECT id, title, note, done, created_at FROM todos';
        var conditions = [];
        var params = [];

        if (filter === 'active') {
            conditions.push('done = 0');
        }
        if (search) {
            conditions.push('(title LIKE ? OR note LIKE ?)');
            params.push('%' + search + '%');
            params.push('%' + search + '%');
        }
        if (conditions.length > 0) {
            sql += ' WHERE ' + conditions.join(' AND ');
        }
        sql += ' ORDER BY done ASC, created_at DESC';

        var rows = skip.db.query(sql, params);
        var items = [];
        for (var i = 0; i < rows.length; i++) {
            var r = rows[i];
            items.push({
                id: r.id,
                title: r.title,
                note: r.note || '',
                done: r.done === 1,
                dateLabel: formatDate(r.created_at)
            });
        }

        // Count labels
        var totalRows = skip.db.query('SELECT COUNT(*) as cnt FROM todos', []);
        var doneRows = skip.db.query('SELECT COUNT(*) as cnt FROM todos WHERE done = 1', []);
        var total = totalRows.length > 0 ? totalRows[0].cnt : 0;
        var done = doneRows.length > 0 ? doneRows[0].cnt : 0;
        var label = total + ' ' + skip.i18n.t('count.total');
        if (done > 0) {
            label += ' \u00B7 ' + done + ' ' + skip.i18n.t('count.completed');
        }

        this.setData({ items: items, countLabel: label });
    }
});
