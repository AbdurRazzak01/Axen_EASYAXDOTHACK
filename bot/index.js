const TelegramBot = require('node-telegram-bot-api');
const { fetchCurrentPrice, generateFakePricesAroundBase, generateChartUrl } = require('./price');
const botToken = '8152846708:AAHMIOpvbL9SWufL4HDu5KEnWdfWIrow1EE';
const bot = new TelegramBot(botToken, { polling: true });

// WebApp URLs
const depositUrl = 'https://botaxen.netlify.app/deposit.html';
const withdrawUrl = 'https://botaxen.netlify.app/withdraw.html';

// Permanent reply keyboard
const mainKeyboard = {
    reply_markup: {
        keyboard: [
          
            ['💰 Deposit', '🏦 Withdraw'],
            ['📈 See Price', '🧠 Subscribe to Strategy']
        ],
        resize_keyboard: true,
        one_time_keyboard: false
    },
    parse_mode: 'Markdown'
};

// Store subscribed users
const subscribedUsers = new Map(); // chatId -> strategyIndex

// Hardcoded strategies
const hardcodedStrategies = [
    "🚀 *Strategy Alert*\n\n📥 Recommend: *Deposit into Axen Vault for maximum profit this Sunday!*.\nRationale: Optimized yield conditions detected across parachains. 📈",
    "⚡ *Strategy Alert*\n\n📤 Action: *Withdraw partial assets from your Axen Vault*.\nRisk Level: Short-term volatility spike on monitored assets. 🛡️",
    "🌐 *Strategy Alert*\n\n🔄 Suggestion: *Maintain current deposits in Axen Vault*.\nReason: Cross-chain liquidity inflows stabilizing positions. 🔗",
    "📈 *Strategy Alert*\n\n📥 Immediate: *Increase your position in Axen Vault*.\nMomentum: Uptrend signals confirmed in yield metrics. 🚀",
    "💎 *Strategy Alert*\n\n💼 Advisory: *Hold assets within Axen Vault*.\nStrategy: Long-term APY and security remain strong. 🛡️",
    "🔥 *Strategy Alert*\n\n⚡ Tactical Move: *Consider partial withdrawal from Axen Vault*.\nOpportunity: Arbitrage windows active across secondary pools. ⏳"
];

// /start command
bot.onText(/\/start/, (msg) => {
    const chatId = msg.chat.id;
    const firstName = msg.from.first_name || 'there';

    console.log('Received /start command');

    const welcomeMessage = `👋 Hello ${firstName}!\n\nWelcome to *Axen*! I'm here to make money for you! Ready to go? 🚀\n\nPlease choose an option below:`;

    bot.sendMessage(chatId, welcomeMessage, mainKeyboard);
});

// Handle text button clicks (from reply keyboard)
bot.on('message', async (msg) => {
    const chatId = msg.chat.id;
    const text = msg.text;

    if (text === '💰 Deposit') {
      bot.sendMessage(chatId, '🔗 *Click below to deposit into Axen Vault*:', {
          parse_mode: 'Markdown',
          reply_markup: {
              inline_keyboard: [
                  [{ text: '🚀 Deposit Now', web_app: { url: depositUrl } }]
              ]
          }
      });
  }
  

  if (text === '🏦 Withdraw') {
    bot.sendMessage(chatId, '🔗 *Click below to withdraw from Axen Vault*:', {
        parse_mode: 'Markdown',
        reply_markup: {
            inline_keyboard: [
                [{ text: '🏦 Withdraw Now', web_app: { url: withdrawUrl } }]
            ]
        }
    });
}


    if (text === '📈 See Price') {
        await sendPrice(chatId);
    }

    if (text === '🧠 Subscribe to Strategy') {
        if (!subscribedUsers.has(chatId)) {
            subscribedUsers.set(chatId, 0);
            bot.sendMessage(chatId, `🧠 You have successfully *subscribed* to strategy notifications!\n\nExpect expert strategies delivered right here! 🚀`, mainKeyboard);
            sendStrategiesSequentially(chatId);
        } else {
            bot.sendMessage(chatId, `✅ You are already subscribed!`, mainKeyboard);
        }
    }
});

// Function to send price chart and current price
async function sendPrice(chatId) {
    try {
        const priceResult = await fetchCurrentPrice('polkadot', 'usd');

        if (!priceResult.success) {
            bot.sendMessage(chatId, '❌ Unable to fetch DOT price at the moment.', mainKeyboard);
            return;
        }

        const basePrice = priceResult.price;
        const { prices, timestamps } = generateFakePricesAroundBase(basePrice, 10);
        const chartUrl = generateChartUrl(timestamps, prices);

        await bot.sendPhoto(chatId, chartUrl, { caption: `📈 Price trend of Axen Vault` });

        await bot.sendMessage(chatId, `💵 *Current DOT Price*: *$${basePrice.toFixed(2)} USD*`, {
            parse_mode: 'Markdown',
            ...mainKeyboard
        });

    } catch (error) {
        console.error('Error fetching price:', error.message);
        bot.sendMessage(chatId, '❌ Error fetching price.', mainKeyboard);
    }
}

// Function to send strategies one-by-one every 30 seconds
function sendStrategiesSequentially(chatId) {
    const currentIndex = subscribedUsers.get(chatId) || 0;

    if (currentIndex >= hardcodedStrategies.length) {
        console.log(`All strategies sent to user ${chatId}`);
        return; // No more strategies left
    }

    // Send current strategy
    bot.sendMessage(chatId, hardcodedStrategies[currentIndex], mainKeyboard);

    console.log(`Sent strategy ${currentIndex} to user ${chatId}`);

    // Update user's next index
    subscribedUsers.set(chatId, currentIndex + 1);

    // Schedule next strategy after 30 seconds
    setTimeout(() => {
        sendStrategiesSequentially(chatId);
    }, 30 * 1000); // 30 seconds
}

