

* Use 

```bash
cargo run test1.json

[{"models":["Fiesta","Focus","Mustang"],"name":"Ford"},{"models":["320","X3","X5"],"name":"BMW"},{"models":["500","Panda"],"name":"Fiat"}]


cargo run test1.json "[0,2]"

{"models":["Fiesta","Focus","Mustang"],"name":"Ford"}
{"models":["500","Panda"],"name":"Fiat"}


Multi cmd

jqr % cargo run --release test1.json "[1,2]|{models}|[0]"


cargo run --release sample-github.json "[0]|{parents}|[0]|{url}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"]), MultiArrayIndex([0]), KeywordAccess(["url"])])
"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"


cargo run --release sample-github.json "[0]|{parents}"
Cmd=MultiCmd([MultiArrayIndex([0]), KeywordAccess(["parents"])])
[{"html_url":"https://github.com/stedolan/jq/commit/9163e09605383a88f6e953d6cb5cc2aebe18c84f","sha":"9163e09605383a88f6e953d6cb5cc2aebe18c84f","url":"https://api.github.com/repos/stedolan/jq/commits/9163e09605383a88f6e953d6cb5cc2aebe18c84f"}]



Now with whitespace 

cargo run --release sample-github.json "[0] | {author} | {url}"


and running on large json
 cargo run --release 004ff2c5-7ed0-433b-8638-e6ceeceb1d09-7 "{Records} | [0] | { Details } | [0]"

 {"CTR":0.007064,"Clicks":7,"Date":"2020-04-24T00:00:00Z","DateType":null,"Details":[],"DuplicateIPBucketClicks":0,"DuplicateImpressionBucketClicks":0,"Events":{},"FirstDate":"2020-04-24T00:00:00Z","Grouping":{"AdTypeId":0,"BrandId":0,"CampaignId":548278,"ChannelId":0,"City":null,"CountryCode":null,"CreativeId":0,"Date":0,"DateType":null,"Keyword":null,"MetroCode":0,"OptionId":11091079,"Price":"0","PriorityId":0,"PublisherAccountId":0,"RateTypeId":2,"Region":null,"SiteId":681017,"ZoneId":0},"Impressions":99092,"InvalidUABucketClicks":0,"LastDate":"2020-04-24T00:00:00Z","RawBucketClicks":7,"Revenue":0.0,"SuspiciousBucketClicks":0,"TestBucketClicks":0,"Title":"UK - Sainsbury's Nectar - Feb 2020 - AdylicPortfolio IO-016066 Companion","TrueRevenue":0.0,"UniqueBucketClicks":7,"UniqueCTR":0.007064,"_title":null,"eCPM":0.0}
```