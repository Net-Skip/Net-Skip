// Weather MiniApp — Logic Layer
// Fetches live weather from Open-Meteo (no API key required).

var CITIES = [
    { id: 'paris',     name: 'Paris',     lat: 48.8566, lon: 2.3522 },
    { id: 'newyork',   name: 'New York',  lat: 40.7128, lon: -74.0060 },
    { id: 'tokyo',     name: 'Tokyo',     lat: 35.6762, lon: 139.6503 },
    { id: 'sydney',    name: 'Sydney',    lat: -33.8688, lon: 151.2093 },
    { id: 'london',    name: 'London',    lat: 51.5074, lon: -0.1278 },
    { id: 'beijing',   name: 'Beijing',   lat: 39.9042, lon: 116.4074 }
];

// WMO weather code to emoji + description key mapping
var WMO_MAP = {
    0:  { icon: '\u2600\uFE0F', key: 'wmo.clear' },
    1:  { icon: '\uD83C\uDF24\uFE0F', key: 'wmo.mostly_clear' },
    2:  { icon: '\u26C5', key: 'wmo.partly_cloudy' },
    3:  { icon: '\u2601\uFE0F', key: 'wmo.overcast' },
    45: { icon: '\uD83C\uDF2B\uFE0F', key: 'wmo.fog' },
    48: { icon: '\uD83C\uDF2B\uFE0F', key: 'wmo.fog' },
    51: { icon: '\uD83C\uDF26\uFE0F', key: 'wmo.drizzle' },
    53: { icon: '\uD83C\uDF26\uFE0F', key: 'wmo.drizzle' },
    55: { icon: '\uD83C\uDF26\uFE0F', key: 'wmo.drizzle' },
    61: { icon: '\uD83C\uDF27\uFE0F', key: 'wmo.rain' },
    63: { icon: '\uD83C\uDF27\uFE0F', key: 'wmo.rain' },
    65: { icon: '\uD83C\uDF27\uFE0F', key: 'wmo.heavy_rain' },
    66: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.freezing_rain' },
    67: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.freezing_rain' },
    71: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.snow' },
    73: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.snow' },
    75: { icon: '\u2744\uFE0F', key: 'wmo.heavy_snow' },
    77: { icon: '\u2744\uFE0F', key: 'wmo.snow_grains' },
    80: { icon: '\uD83C\uDF26\uFE0F', key: 'wmo.showers' },
    81: { icon: '\uD83C\uDF27\uFE0F', key: 'wmo.showers' },
    82: { icon: '\u26C8\uFE0F', key: 'wmo.heavy_showers' },
    85: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.snow_showers' },
    86: { icon: '\uD83C\uDF28\uFE0F', key: 'wmo.snow_showers' },
    95: { icon: '\u26C8\uFE0F', key: 'wmo.thunderstorm' },
    96: { icon: '\u26C8\uFE0F', key: 'wmo.thunderstorm_hail' },
    99: { icon: '\u26C8\uFE0F', key: 'wmo.thunderstorm_hail' }
};

function wmoInfo(code) {
    return WMO_MAP[code] || { icon: '\u2753', key: 'wmo.unknown' };
}

function cToF(c) { return (c * 9 / 5) + 32; }

function formatTemp(c, unit) {
    if (unit === 'F') return Math.round(cToF(c)) + '\u00B0F';
    return Math.round(c) + '\u00B0C';
}

function dayName(dateStr, lang) {
    var d = new Date(dateStr + 'T12:00:00');
    var days_en = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    var days_fr = ['Dim', 'Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam'];
    var days_zh = ['\u5468\u65E5', '\u5468\u4E00', '\u5468\u4E8C', '\u5468\u4E09', '\u5468\u56DB', '\u5468\u4E94', '\u5468\u516D'];
    var dow = d.getDay();
    if (lang === 'fr') return days_fr[dow];
    if (lang === 'zh-CN' || lang === 'zh') return days_zh[dow];
    return days_en[dow];
}

Page({
    data: {
        cities: CITIES,
        selectedCity: 'paris',
        cityName: 'Paris',
        coords: '',
        unit: 'C',
        loading: true,
        error: '',
        currentTemp: '--',
        currentIcon: '',
        currentDesc: '',
        humidity: '--',
        wind: '--',
        forecast: []
    },

    onLoad: function() {
        skip.log('Weather page loaded');
        this.fetchWeather('paris');
    },

    onSelectCity: function(event) {
        var cityId = event.detail.id;
        skip.log('Selected city: ' + cityId);
        this.setData({ selectedCity: cityId });
        this.fetchWeather(cityId);
    },

    onToggleUnit: function(event) {
        var unit = event.detail.unit;
        skip.log('Switched to ' + unit);
        this.setData({ unit: unit });
        // Re-render temperatures from cached raw data
        if (this._rawData) {
            this.applyWeatherData(this._rawData, unit);
        }
    },

    fetchWeather: async function(cityId) {
        var city = null;
        for (var i = 0; i < CITIES.length; i++) {
            if (CITIES[i].id === cityId) { city = CITIES[i]; break; }
        }
        if (!city) return;

        this.setData({
            loading: true,
            error: '',
            cityName: city.name,
            coords: city.lat.toFixed(2) + ', ' + city.lon.toFixed(2)
        });

        var url = 'https://api.open-meteo.com/v1/forecast'
            + '?latitude=' + city.lat
            + '&longitude=' + city.lon
            + '&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m'
            + '&daily=weather_code,temperature_2m_max,temperature_2m_min'
            + '&timezone=auto&forecast_days=5';

        skip.log('Fetching: ' + url);

        try {
            var response = await skip.fetch(url);
            if (!response.ok) {
                this.setData({ loading: false, error: skip.i18n.t('weather.error') + ' (' + response.status + ')' });
                return;
            }
            var data = await response.json();
            skip.log('Weather data received for ' + city.name);
            this._rawData = data;
            this.applyWeatherData(data, this.data.unit);
        } catch (e) {
            skip.log('Fetch error: ' + e.message);
            this.setData({ loading: false, error: skip.i18n.t('weather.error') + ': ' + e.message });
        }
    },

    applyWeatherData: function(data, unit) {
        var current = data.current;
        var daily = data.daily;
        var info = wmoInfo(current.weather_code);
        var lang = 'en';
        try { lang = skip.i18n.getLocale ? skip.i18n.getLocale() : 'en'; } catch(e) {}

        var windUnit = (unit === 'F') ? ' mph' : ' km/h';
        var windVal = current.wind_speed_10m;
        if (unit === 'F') windVal = Math.round(windVal * 0.621371);
        else windVal = Math.round(windVal);

        // Build forecast array
        var forecastArr = [];
        for (var i = 0; i < daily.time.length; i++) {
            var dInfo = wmoInfo(daily.weather_code[i]);
            forecastArr.push({
                name: i === 0 ? skip.i18n.t('forecast.today') : dayName(daily.time[i], lang),
                icon: dInfo.icon,
                hi: formatTemp(daily.temperature_2m_max[i], unit),
                lo: formatTemp(daily.temperature_2m_min[i], unit)
            });
        }

        this.setData({
            loading: false,
            error: '',
            currentTemp: formatTemp(current.temperature_2m, unit),
            currentIcon: info.icon,
            currentDesc: skip.i18n.t(info.key),
            humidity: current.relative_humidity_2m + '%',
            wind: windVal + windUnit,
            forecast: forecastArr
        });
    }
});
