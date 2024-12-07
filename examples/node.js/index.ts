import {
  SQSClient,
  GetQueueUrlCommand,
  ReceiveMessageCommand,
  SendMessageCommand,
  CreateQueueCommand,
} from "@aws-sdk/client-sqs";

const hostUrl = "http://localhost:8080/sqs";

const sqs = new SQSClient({
  endpoint: hostUrl,
  region: "us-west-1",
  credentials: {
    accessKeyId: "6kkMWFC1nin",
    secretAccessKey: "FhwbQ682XAe7PxcY7WWkJKGscqdpdknZP",
  },
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
