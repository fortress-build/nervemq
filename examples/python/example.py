import boto3
from types_boto3_sqs import SQSClient

host_url = 'http://localhost:8080/sqs'


def main():
    sqs: SQSClient = boto3.client(
        'sqs',
        aws_access_key_id='PdNJWCvVtcc',
        aws_secret_access_key='JosFLEGqsRdmQ4hE1ppVEZ1qKV3M3bVFN',
        region_name='us-west-1',
        endpoint_url=host_url,
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
        MessageAttributeNames=['Test'],
        MaxNumberOfMessages=10,
        VisibilityTimeout=0,
        WaitTimeSeconds=0,
        ReceiveRequestAttemptId='1',
    )

    print(f'Messages: {response.get("Messages")}')
