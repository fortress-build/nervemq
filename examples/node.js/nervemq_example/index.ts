import { SQSClient, GetQueueUrlCommand, CreateQueueCommand, SendMessageCommand, ReceiveMessageCommand } from "@aws-sdk/client-sqs";

const hostUrl = 'http://localhost:8080/sqs';

async function main() {
    const sqs = new SQSClient({
        endpoint: hostUrl,
        region: "us-west-1",
        credentials: {
            accessKeyId: "ZBcnTSTKX69",
            secretAccessKey: "38zBir4Vvn2SKAx6VpPAvdNY4LzBaBGBQ"
        }
    });

    // biome-ignore lint/suspicious/noImplicitAnyLet: <explanation>
    let url;
    try {
        const res = await sqs.send(new GetQueueUrlCommand({
            QueueName: 'bruh'
        }));
        url = res.QueueUrl;
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
        const res = await sqs.send(new CreateQueueCommand({
            QueueName: 'bruh'
        }));
        url = res.QueueUrl;
    }

    console.log(`Queue URL: ${url}`);

    const sendResult = await sqs.send(new SendMessageCommand({
        QueueUrl: url,
        MessageBody: 'Hello World!',
        MessageAttributes: {
            'Test': {
                StringValue: 'TestString',
                DataType: 'String'
            }
        }
    }));

    console.log(`Message ID: ${sendResult.MessageId}`);

    const receiveResult = await sqs.send(new ReceiveMessageCommand({
        QueueUrl: url,
        AttributeNames: ['All'],
        MessageAttributeNames: ['Test'],
        MaxNumberOfMessages: 10,
        VisibilityTimeout: 0,
        WaitTimeSeconds: 0,
        ReceiveRequestAttemptId: '1'
    }));

    console.log(`Messages: ${JSON.stringify(receiveResult.Messages)}`);
}

main().catch(console.error);
