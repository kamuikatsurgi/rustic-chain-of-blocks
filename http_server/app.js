import express, { json } from 'express';
import fs from 'fs';
import cors from 'cors';
import createKeccakHash from 'keccak'

const app = express();

const corsOption = {
    origin:'*',
    credentials:true,
    optionSuccessStatus:200,
};

app.use(express.json());

app.use(cors(corsOption));

app.get("/blockNumber", (req, res) => {
    fs.readFile('../blockchain.json', 'utf8', (err, data) => {
        if (err) {
            console.error('Error reading JSON file:', err);
            res.status(500).json({ error: 'Internal Server Error' });
            return;
        }

        try {
            const blockchain = JSON.parse(data);
            const blocks = blockchain.blocks;
 
            let highestBlock = blocks[0];
            for (let i = 1; i < blocks.length; i++) {
                if (blocks[i].header.number > highestBlock.header.number) {
                    highestBlock = blocks[i];
                }
            }

            res.json(highestBlock.header.number);
        } catch (error) {
            console.error('Error parsing JSON:', error);
            res.status(500).json({ error: 'Internal Server Error' });
        }
    });
});

app.get('/block', (req, res) => {
    const blockHash = req.query.hash;
    const blockNumber = parseInt(req.query.number, 10);
    
    if (!blockNumber && !blockHash) {
        return res.status(400).json({ error: 'Please provide either blockNumber or blockHash query parameter' });
    }

    if (blockNumber && blockHash) {
        return res.status(400).json({ error: 'Please provide only one of blockNumber or blockHash query parameter' });
    }
    
    fs.readFile('../blockchain.json', 'utf8', (err, data) => {
        if (err) {
            console.error('Error reading JSON file:', err);
            res.status(500).json({ error: 'Internal Server Error' });
            return;
        }

        try {

            const blockchain = JSON.parse(data);
            const blocks = blockchain.blocks;

            if (!blocks || blocks.length === 0) {
                return res.json(null);
            }
            
            if (blockNumber) {
                const block = blocks.find(block => block.header.number === blockNumber);
                res.json(block || null);
            }

            if (blockHash) {
                const block = blocks.find(block => calculateBlockHash(block) === blockHash);
                res.json(block || null);
            }

        } catch (error) {
            console.error('Error parsing JSON:', error);
            res.status(500).json({ error: 'Internal Server Error' });
        }
    });
});

function calculateBlockHash(block) {
    const header = block.header;
    const extraDataBytes = JSON.stringify(header.extra_data).split('').map(c => c.charCodeAt(0));
    const txsBytes = JSON.stringify(block.txs).split('').map(c => c.charCodeAt(0));

    const hash = createKeccakHash('keccak256')
        .update(header.parent_hash)
        .update(header.miner)
        .update(header.state_root)
        .update(header.transactions_root)
        .update(header.difficulty.toString())
        .update(header.total_difficulty.toString())
        .update(header.number.toString())
        .update(header.timestamp.toString())
        .update(header.nonce.toString())
        .update(Buffer.from(extraDataBytes))
        .update(Buffer.from(txsBytes))
        .digest('hex');

    return `0x${hash}`;
}

app.listen(8888, () => {
    console.log('Server is running on port 8888');
})