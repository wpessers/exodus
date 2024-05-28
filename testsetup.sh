#!/bin/bash

# Make sure docker desktop is running if you're using WSL
localstack start -d

until [ "`docker inspect -f {{.State.Running}} localstack-main`"=="true" ]; do
    sleep 0.1;
done;

export AWS_ACCESS_KEY_ID=000000000001
export AWS_SECRET_ACCESS_KEY=000000000001
export AWS_SESSION_TOKEN=000000000001
export AWS_REGION=eu-west-1


# Api keys unrelated to any usage plans
awslocal apigateway create-api-key --name "Kevin's key for testing" --region eu-west-1 > /dev/null


# Dev keys for the cookie API
cookieDevKeyId1=$(awslocal apigateway create-api-key --name "TestKey - dev - Cookie" --region eu-west-1 | jq -r .id)
cookieDevKeyId2=$(awslocal apigateway create-api-key --name "TestKey - Development - Chocolate Chip Cookie" --region eu-west-1 | jq -r .id)

cookieDevUsagePlanId=$(awslocal apigateway create-usage-plan --name cookie-plan-dev --region eu-west-1 | jq -r .id)
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieDevUsagePlanId --key-id $cookieDevKeyId1 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieDevUsagePlanId --key-id $cookieDevKeyId2 --key-type API_KEY --region eu-west-1 > /dev/null


# NonProd keys for the cookie API
cookieNonProdKeyId1=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId2=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Chocolate Chip Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId3=$(awslocal apigateway create-api-key --name "TestKey - non-prod - White Chocolate Chip Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId4=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Triple Chocolate Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId5=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Undercooked Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId6=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Uranium Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId7=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Blueberry Cookie" --region eu-west-1 | jq -r .id)
cookieNonProdKeyId8=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Pistachio Cookie" --region eu-west-1 | jq -r .id)

cookieNonProdUsagePlanId=$(awslocal apigateway create-usage-plan --name cookie-plan-nonprod --region eu-west-1 | jq -r .id)
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId1 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId2 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId3 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId4 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId5 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId6 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId7 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $cookieNonProdUsagePlanId --key-id $cookieNonProdKeyId8 --key-type API_KEY --region eu-west-1 > /dev/null


# Usage plan without any api keys
cookieProdUsagePlanId=$(awslocal apigateway create-usage-plan --name cookie-plan-prod --region eu-west-1 | jq -r .id)


# Nonprod keys for the pancake api
pancakeNonProdKeyId1=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Pancake" --region eu-west-1 | jq -r .id)
pancakeNonProdKeyId2=$(awslocal apigateway create-api-key --name "TestKey - non-prod - Fluffy pancake" --region eu-west-1 | jq -r .id)
pancakeNonProdKeyId3=$(awslocal apigateway create-api-key --name "TestKey - NonProd - French toast" --region eu-west-1 | jq -r .id)
pancakeNonProdKeyId4=$(awslocal apigateway create-api-key --name "TestKey - non prod - Crepe" --region eu-west-1 | jq -r .id)
pancakeNonProdKeyId5=$(awslocal apigateway create-api-key --name "TestKey - nonprod - Uranium pancake" --region eu-west-1 | jq -r .id)

pancakeUsagePlanId=$(awslocal apigateway create-usage-plan --name pancake-plan-nonprod --region eu-west-1 | jq -r .id)
awslocal apigateway create-usage-plan-key --usage-plan-id $pancakeUsagePlanId --key-id $pancakeNonProdKeyId1 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $pancakeUsagePlanId --key-id $pancakeNonProdKeyId2 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $pancakeUsagePlanId --key-id $pancakeNonProdKeyId3 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $pancakeUsagePlanId --key-id $pancakeNonProdKeyId4 --key-type API_KEY --region eu-west-1 > /dev/null
awslocal apigateway create-usage-plan-key --usage-plan-id $pancakeUsagePlanId --key-id $pancakeNonProdKeyId5 --key-type API_KEY --region eu-west-1 > /dev/null
