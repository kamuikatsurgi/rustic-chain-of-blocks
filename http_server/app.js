import express, { json } from "express";
import fs from "fs";
import cors from "cors";
import createKeccakHash from "keccak";

const app = express();

const corsOption = {
  origin: "*",
  credentials: true,
  optionSuccessStatus: 200,
};

app.use(express.json());

app.use(cors(corsOption));

app.get("/blockNumber", (req, res) => {
  fs.readFile("../blockchain.json", "utf8", (err, data) => {
    if (err) {
      console.error("Error reading JSON file:", err);
      res.status(500).json({ error: "Internal Server Error" });
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
      console.error("Error parsing JSON:", error);
      res.status(500).json({ error: "Internal Server Error" });
    }
  });
});

app.get("/block", (req, res) => {
  const blockHash = req.query.hash;
  const blockNumber = parseInt(req.query.number, 10);

  if (!blockNumber && !blockHash) {
    return res.status(400).json({
      error: "Please provide either blockNumber or blockHash query parameter",
    });
  }

  if (blockNumber && blockHash) {
    return res.status(400).json({
      error:
        "Please provide only one of blockNumber or blockHash query parameter",
    });
  }

  fs.readFile("../blockchain.json", "utf8", (err, data) => {
    if (err) {
      console.error("Error reading JSON file:", err);
      res.status(500).json({ error: "Internal Server Error" });
      return;
    }

    try {
      const blockchain = JSON.parse(data);
      const blocks = blockchain.blocks;

      if (!blocks || blocks.length === 0) {
        return res.json(null);
      }

      if (blockNumber) {
        const block = blocks.find(
          (block) => block.header.number === blockNumber
        );
        res.json(block || null);
      }

      if (blockHash) {
        const block = blocks.find(
          (block) => calculateBlockHash(block) === blockHash
        );
        res.json(block || null);
      }
    } catch (error) {
      console.error("Error parsing JSON:", error);
      res.status(500).json({ error: "Internal Server Error" });
    }
  });
});

app.get("/tx", (req, res) => {
  const txHash = req.query.hash;

  if (!txHash) {
    return res
      .status(400)
      .json({ error: "Please provide a transaction hash query parameter" });
  }

  fs.readFile("../blockchain.json", "utf8", (err, data) => {
    if (err) {
      console.error("Error reading JSON file:", err);
      res.status(500).json({ error: "Internal Server Error" });
      return;
    }

    try {
      const blockchain = JSON.parse(data);
      const blocks = blockchain.blocks;

      if (!blocks || blocks.length === 0) {
        return res.json(null);
      }

      const foundTransaction = blocks.reduce((foundTx, block) => {
        if (foundTx) return foundTx;
        return block.txs.find((tx) => calculateTransactionHash(tx) === txHash);
      }, null);

      if (!foundTransaction) {
        return res.status(404).json({ error: "Transaction not found" });
      }

      res.json(foundTransaction);
    } catch (error) {
      console.error("Error parsing JSON:", error);
      res.status(500).json({ error: "Internal Server Error" });
    }
  });
});

app.get("/getNonce", (req, res) => {
  const address = req.query.address;

  if (!address) {
    return res.status(400).json({
      error: "Please provide address of the account that you want to query",
    });
  }

  fs.readFile("../accounts.json", "utf8", (err, data) => {
    if (err) {
      console.error("Error reading JSON file:", err);
      res.status(500).json({ error: "Internal Server Error" });
      return;
    }

    try {
      const accounts = JSON.parse(data);

      if (!accounts.length === 0) {
        return res.json(null);
      }

      if (address) {
        const account = accounts.find((acc) => acc.address === address);
        if (!account) {
          res.json(null);
        } else {
          res.json(account.nonce);
        }
      }
    } catch (error) {
      console.error("Error parsing JSON:", error);
      res.status(500).json({ error: "Internal Server Error" });
    }
  });
});

app.get("/getBalance", (req, res) => {
  const address = req.query.address;

  if (!address) {
    return res.status(400).json({
      error: "Please provide address of the account that you want to query",
    });
  }

  fs.readFile("../accounts.json", "utf8", (err, data) => {
    if (err) {
      console.error("Error reading JSON file:", err);
      res.status(500).json({ error: "Internal Server Error" });
      return;
    }

    try {
      const accounts = JSON.parse(data);

      if (!accounts.length === 0) {
        return res.json(null);
      }

      if (address) {
        const account = accounts.find((acc) => acc.address === address);
        if (!account) {
          res.json(null);
        } else {
          res.json(account.balance);
        }
      }
    } catch (error) {
      console.error("Error parsing JSON:", error);
      res.status(500).json({ error: "Internal Server Error" });
    }
  });
});

app.post("/sendTx", (req, res) => {
  const { from, to, value, pk } = req.body;

  if (!from || !to || !value || !pk) {
    return res.status(400).json({
      error:
        "Please provide from, to, value, and privateKey in the request body",
    });
  }

  const mempoolFilePath = "../mempool.json";
  const txData = {
    from: from,
    to: to,
    value: value,
    pk: pk,
  };

  try {
    const mempoolData = fs.readFileSync(mempoolFilePath);
    const mempool = JSON.parse(mempoolData);
    mempool.push(txData);
    fs.writeFileSync(mempoolFilePath, JSON.stringify(mempool, null, 2));
  } catch (err) {
    console.error("Error appending transaction to mempool:", err);
    return res.status(500).json({ error: "Internal server error" });
  }

  res.status(201).json({ message: "Transaction added to mempool" });
});

function calculateBlockHash(block) {
  const header = block.header;
  const extraDataBytes = JSON.stringify(header.extra_data)
    .split("")
    .map((c) => c.charCodeAt(0));
  const txsBytes = JSON.stringify(block.txs)
    .split("")
    .map((c) => c.charCodeAt(0));

  const hash = createKeccakHash("keccak256")
    .update(header.parent_hash)
    .update(header.miner)
    .update(header.state_root)
    .update(header.transactions_root)
    .update(header.number.toString())
    .update(header.timestamp.toString())
    .update(Buffer.from(extraDataBytes))
    .update(Buffer.from(txsBytes))
    .digest("hex");

  return `0x${hash}`;
}

function calculateTransactionHash(tx) {
  const hash = createKeccakHash("keccak256")
    .update(tx.sender)
    .update(tx.receiver)
    .update(tx.value.toString())
    .update(tx.nonce.toString())
    .update(tx.v)
    .update(tx.r)
    .update(tx.s)
    .digest("hex");

  return hash;
}

app.listen(8888, () => {
  console.log("Server is running on port 8888");
});
