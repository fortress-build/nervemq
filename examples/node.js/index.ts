import {
  SQSClient,
  GetQueueUrlCommand,
  ReceiveMessageCommand,
  SendMessageCommand,
  CreateQueueCommand,
} from "@aws-sdk/client-sqs";

const endpoint = "http://localhost:8080/sqs";
const region = "some-fake-region";

const {
  AWS_ACCESS_KEY_ID: accessKeyId,
  AWS_SECRET_ACCESS_KEY: secretAccessKey,
} = process.env;

if (accessKeyId === undefined || secretAccessKey === undefined) {
  throw new Error("AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY must be set");
}

const sqs = new SQSClient({
  endpoint,
  region,
  credentials: { accessKeyId, secretAccessKey },
});

const url = await sqs
  .send(new GetQueueUrlCommand({ QueueName: "bruh" }))
  .catch(() => sqs.send(new CreateQueueCommand({ QueueName: "bruh" })))
  .then((res) => res.QueueUrl);

console.log(`Queue URL: ${url}`);

const sendResult = await sqs.send(
  new SendMessageCommand({
    QueueUrl: url,
    MessageBody: "Hello World!",
    MessageAttributes: {
      Test: {
        StringValue: "TestString",
        DataType: "String",
      },
    },
  }),
);

console.log(`Message ID: ${sendResult.MessageId}`);

const receiveResult = await sqs.send(
  new ReceiveMessageCommand({
    QueueUrl: url,
    MessageAttributeNames: ["Test"],
  }),
);

console.log(`Messages: ${JSON.stringify(receiveResult.Messages)}`);
