const axios = require('axios');

// CoinGecko simple price endpoint
const COINGECKO_SIMPLE_PRICE = 'https://api.coingecko.com/api/v3/simple/price';

// Fetch real current DOT price
async function fetchCurrentPrice(coinId = 'polkadot', vsCurrency = 'usd') {
    try {
        const response = await axios.get(COINGECKO_SIMPLE_PRICE, {
            params: {
                ids: coinId,
                vs_currencies: vsCurrency
            }
        });

        const price = response.data[coinId][vsCurrency];
        return {
            success: true,
            price
        };

    } catch (error) {
        console.error('Error fetching current price:', error.message);
        return { success: false, message: 'Failed to fetch current price' };
    }
}

// Generate small fake fluctuations around current real price
function generateFakePricesAroundBase(basePrice, points = 10) {
    const prices = [parseFloat(basePrice.toFixed(2))];

    for (let i = 1; i < points; i++) {
        const lastPrice = prices[i - 1];
        const fluctuation = (Math.random() - 0.5) * 0.01; // ±0.5% fake fluctuation
        const newPrice = lastPrice * (1 + fluctuation);
        prices.push(parseFloat(newPrice.toFixed(2)));
    }

    const timestamps = Array.from({ length: points }, (_, i) => `${i}:00`);

    return { prices, timestamps };
}

// Generate chart URL using QuickChart
function generateChartUrl(timestamps, prices) {
    const chartConfig = {
        type: 'line',
        data: {
            labels: timestamps,
            datasets: [{
                label: 'DOT Simulated Trend',
                data: prices,
                fill: false,
                borderColor: 'rgb(75, 192, 192)',
                tension: 0.3
            }]
        },
        options: {
            scales: {
                x: {
                    ticks: { color: 'white' }
                },
                y: {
                    ticks: { color: 'white' }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: 'white'
                    }
                }
            }
        }
    };

    const encodedConfig = encodeURIComponent(JSON.stringify(chartConfig));
    return `https://quickchart.io/chart?c=${encodedConfig}`;
}

module.exports = {
    fetchCurrentPrice,
    generateFakePricesAroundBase,
    generateChartUrl
};
