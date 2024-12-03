import boto3
from types_boto3_sqs import SQSClient

custom_url = 'http://localhost:8080/sqs'


def main():
    sqs: SQSClient = boto3.client(
        'sqs',
        aws_access_key_id='UyTz3t56rjb',
        aws_secret_access_key='GoLqobpKZiKyvdrGB5jmbmTizHsC12cXH',
        region_name='us-west-1',
        endpoint_url=custom_url,
    )

    url = None
    try:
        res = sqs.get_queue_url(QueueName='bruh')
        url = res.get('QueueUrl')
    except:
        res = sqs.create_queue(QueueName='bruh')
        url = res.get('QueueUrl')

    print(f'Queue URL: {url}')

    response = sqs.send_message(
        QueueUrl=url,
        MessageBody='Hello World!',
        MessageAttributes={
            'Test': {'StringValue': 'TestString', 'DataType': 'String'}
        },
    )

    print(f'Message ID: {response.get("MessageId")}')

    response = sqs.receive_message(
        QueueUrl=url,
        AttributeNames=['All'],
        MessageAttributeNames=['All'],
        MaxNumberOfMessages=10,
        VisibilityTimeout=0,
        WaitTimeSeconds=0,
        ReceiveRequestAttemptId='1',
    )

    print(f'Messages: {response.get("Messages")}')
