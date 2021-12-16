# DeliveryTracker

## 국내 지원 택배사

|이름|ID|비고|
|---|---|---|
|천일택배|`kr.chunil`|시간 정보 미제공|
|CJ대한통운|`kr.cjlogistics`||
|CU Post|`kr.cupost`||
|대신택배|`kr.daesin`||
|우체국|`kr.epost`||
|우체국 EMS|`kr.epostems`||
|GS Postbox 택배|`kr.gspostbox`||
|한진택배|`kr.hanjin`||
|일양로지스|`kr.ilyanglogis`||
|경동택배|`kr.kyoungdong`||
|로젠택배|`kr.logen`||
|롯데택배|`kr.lotte`||

## 국외 지원 택배사

|이름|ID|비고|
|---|---|---|
|CAINIAO|`cn.cainiao`|한번씩 데이터가 나오지 않음|
|WarpEX|`us.warpex`|

## 택배 상세 정보 메시지 타입 종류

```rust
TrackingDetail {
    time: String,
    message: Some(String),
    status: Some(String),
    location: Some(String),
}
```

```rust
TrackingDetail {
    time: String,
    message: None,
    status: Some(String),
    location: Some(String),
}
```

```rust
TrackingDetail {
    time: String,
    message: Some(String),
    status: None,
    location: Some(String),
}
```

```rust
TrackingDetail {
    time: String,
    message: Some(String),
    status: None,
    location: None,
}
```
