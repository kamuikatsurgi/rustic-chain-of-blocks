import express, { json } from 'express';
import cors from 'cors';

const app = express();

const corsOption = {
    origin:'*',
    credentials:true,
    optionSuccessStatus:200,
};

app.use(express.json());

app.use(cors(corsOption));

app.get("/echo", (req, res) => {
    res.send("Hello World");
});

app.listen(8888, () => {
    console.log('Server is running on port 8888');
})