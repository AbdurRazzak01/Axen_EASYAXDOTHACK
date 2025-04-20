const { ApiPromise, WsProvider } = require('@polkadot/api');

async function testConnection() {
  const provider = new WsProvider('wss://westend-rpc.polkadot.io'); // Connect to Polkadot network
  const api = await ApiPromise.create({ provider });

  try {
    const chain = await api.rpc.system.chain();
    console.log(`Successfully connected to ${chain}`);
    
    const blockNumber = await api.rpc.chain.getBlockHash();
    console.log(`Latest block number: ${blockNumber.toString()}`);
    
    const systemName = await api.rpc.system.name();
    console.log(`System name: ${systemName}`);
    
  } catch (err) {
    console.error('Failed to connect to the Polkadot network:', err);
  }
}

// Call the async function
testConnection();
