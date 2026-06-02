# 自研客服系统 IM —— LBHK iOS App 端内适配 技术方案

> 对接需求:`【需求】自研客服系统项目_2.2_ 自研客服系统 IM LBHK App 端内适配.pdf` (V1.0.0,2026/4/27,陈思宇)
> 目标版本:LBHK iOS,预计 2026-06-22 端内灰度
> 当前分支:`users/baiyang/2026/0518/lbhk/feat/web_custom_service`
>
> **本文档约定:** 所有未与对应方(后端 / 产品 / 数据 / H5)对齐的事项,在文中显式标注「**TBD — 待 XX 对齐**」,不预设猜测命名;阅读者应将 TBD 视为联调期阻塞项。

---

## 一、Context(背景)

公司现有客服 IM 由三方系统「七鱼(QYSDK)」承载;自研客服 IM 的 H5 第一版已完成(测试 URL `https://app.longbridge.xyz/csp/chat`)。本次需求是把自研 IM **以可控灰度的方式**引入 LBHK iOS App,逐步替换七鱼,关键约束如下:

- 命中灰度 → 走自研 IM(H5 容器);未命中 → 走老的七鱼(H5 / 原生)。**全 App 所有客服入口跟随同一灰度逻辑**
- 协议变更:即使 PWA 已同意,App 内首次进入仍需重新弹协议(由 H5 端处理,Native 不参与协议状态判断)
- 登录态注入,无登录中断;退出/切换账号即时关闭 IM 会话
- 未读数(消息中心铃铛、客服条目)、推送(合并 30s / 单日 5 条上限 / 静默时段 / 脱敏)、附件(相册/相机/文件)、暗夜模式适配
- 全程具备一键回滚能力
- **服务端职责确认:30 秒合并、单会话单日 5 条上限、静默时段、敏感信息脱敏 全部由服务端承担,客户端不参与**

---

## 二、技术分析

### 2.1 现有客服入口架构(已收敛)

工程内**所有「联系客服」入口已统一走路由** `kLBCustomerService = lb://page/mine/customer_service`,定义于 [MineRouterURL.h:71](longbridge-ios-app/LBPod/lbmoduleservice/LBModuleService/LBRouterURL/MineRouterURL.h#L71)。

实际分发逻辑在 [QiyuHelper.m:80-103](longbridge-ios-app/LBPod/LBExtraBizKit/LBExtraBizKit/Classes/QiYuHelper/QiyuHelper.m#L80-L103):

```objc
[LBRouter registerURLPattern:kLBCustomerService toModelHandler:^(LBRouterModel *m) {
    // 1. 读租户 customer_service_URL_v2(H5)
    // 2. 读租户 customer_service_URL(H5)
    // 3. fallback → kLBNativeCustomerService(七鱼原生)
}];
```

工程内调用入口约 8 处(消息中心、我的页、资金记录详情、基金交易、股票交易异常 4 处、登录验证、银行卡、资产账户状态等)。**入口完整性 review 由 PDF 指定的负责人(白杨、文博)另行进行,不在本方案范围内**。

> **结论:** 只需在 `kLBCustomerService` 路由 handler 最前面注入「自研 IM 命中」分支,即可让所有入口同步切换。这是本方案最大的杠杆点。

### 2.2 灰度/租户能力(已具备)

- 灰度框架 [FeatureFlagUtil.m:595-601](longbridge-ios-app/LBPod/lbusercenterbizkit/LBUserBizKit/LBConfig/FeatureFlagUtil/FeatureFlagUtil.m#L595-L601):同步 API `+ (BOOL)featureFlagForKey:(NSString *)key`,本地 MMKV 缓存,App 启动后异步刷新
- 灰度 key **必须**先在 [FeatureFlagUtil.m:85-169](longbridge-ios-app/LBPod/lbusercenterbizkit/LBUserBizKit/LBConfig/FeatureFlagUtil/FeatureFlagUtil.m#L85-L169) 的 `featureFlagRequest` keys 数组里**注册**才会被下发到客户端
- 租户开关:`[LBTenantConfigCenter functionSwitchForKey:@"key"]`(BOOL);租户资源:`[LBTenantConfigCenter tenantResourceModelForKey:@"key"]`(NSString)
- 品牌(account channel)判断:`UserDataCenter.currentAccountChannel` 与 `kAccountChannel_LBSG` 比较

### 2.3 H5 容器能力(已就绪,无需新建)

- WebView 容器:[LBWKWebView](longbridge-ios-app/LBPod/LBWebKit/lbwebview/Classes/LBWKWebView.h) + LBWebView/LBWebServiceImpl,通过 `[LBRouter openURL:url]` 直接打开
- Cookie/Token 注入:[WKWebView+LBCookies.m](longbridge-ios-app/LBPod/LBWebKit/lbwebview/Classes/Category/WKWebView+LBCookies.m) `lbUpdateCookie:`,登录态由 [LBWebViewConfigManager.m](longbridge-ios-app/LBPod/LBWebViewModule/LBWebViewModule/Classes/Manager/LBWebViewConfigManager.m) 监听 `UserDataCenter` 自动同步
- **现有 JSBridge API 类共 17 个**(`LBPod/LBWebViewModule/LBWebViewModule/Classes/JSBridgeApi/CommonApis/`):

| 序号 | 类 | 主要能力 |
|---|---|---|
| 1 | LBCacheDataJSBridgeApi | 缓存读写 |
| 2 | LBDefaultJSBridgeApi | Token、JS 执行等通用 |
| 3 | LBEventTrackingJSBridgeApi | 埋点上报 |
| 4 | LBFileJSBridgeApi | 文件读写/选择 |
| 5 | LBIAPJSBridgeApi | 内购 |
| 6 | LBKeyboardJSBridgeApi | 键盘控制 |
| 7 | LBMediaJSBridgeApi | 相册、相机、上传 |
| 8 | LBMyInfoOAuthJSBridgeApi | 用户信息 / OAuth |
| 9 | LBNavigatorJSBridgeApi | 导航栏、进度条 |
| 10 | LBNetDiagJSBridgeApi | 网络诊断 |
| 11 | LBNetworkJSBridgeApi | 网络请求 |
| 12 | LBOpenJSBridgeApi | 通用打开 |
| 13 | LBPermissionJSBridgeApi | 权限申请 |
| 14 | LBRouterJSBridgeApi | 页面跳转、关闭 WebView |
| 15 | LBTradelinkJSBridgeApi | 交易跳转 |
| 16 | LBWSJSBridgeApi | 长连接 |
| 17 | LBZOLOZJSBridgeApi | 三方鉴权 |

**本方案不新增任何 JSBridge API 类。** 自研 IM 所需能力(打埋点、上传图片/文件、申请相册相机权限、关闭 WebView、跳登录页等)全部从现有 17 个里选用。

### 2.4 推送/长连接/消息中心(已就绪)

- APNs 入口 [LBAlicloudPushInitializer.m](longbridge-ios-app/LBPod/LBAppEntry/LBAppEntry/Forwarder/LBAlicloudPushInitializer.m),`handleiOS10Notification:` 中按 payload `link` 字段 `[LBRouter openURL:link]` 路由
- 长连接 [LBQWSClient/LBQWSTCPClient](longbridge-ios-app/LBPod/LBQWSClient/LBQWSClient/Classes/LBQWSClientKit.h),支持自定义 topic 订阅
- 消息中心客服条目 [MessageCenterViewController.m:359-365](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/Controller/MessageCenterViewController.m#L359-L365)(`group_id = -101`),未读数当前订阅 `QiyuHelper.allUnreadCountSubject`
- 铃铛 badge 聚合类 [NewMineBadgeManager.h](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/LBNavView/NewMineBadgeManager.h)
- 退出登录:`<LBUserDataCenterDelegate>.userDataCenter:userLoginStateChanged:automatically:`(七鱼已挂在该回调下登出)

### 2.5 埋点(已就绪)

- 入口 [LBAnalysisHelper](longbridge-ios-app/LBPod/lbanalysis/LBAnalysis/Classes/SensorsAnalysisHelper/LBAnalysisHelper.h):`- (void)track:(NSString *)event withProperties:(NSDictionary *)props`
- H5 端打埋点复用 [LBEventTrackingJSBridgeApi](longbridge-ios-app/LBPod/LBWebViewModule/LBWebViewModule/Classes/JSBridgeApi/CommonApis/LBEventTrackingJSBridgeApi.h)
- 命名规范:事件 + 参数均为 `snake_case`

---

## 三、方案选型(已确认决策)

| 决策点 | 结论 | 备注 |
|---|---|---|
| **入口分发改造点** | 仅改 `kLBCustomerService` 路由 handler,新增「自研 IM 命中」分支放在最前面 | 现有所有入口已收敛到这一个路由,改一处即全 App 切换 |
| **灰度 key** | 复用 PDF 给定的 `customer_service_hk`,**仅一个 key**;品牌差异由后台下发逻辑控制(LBSG 用户后台不下发命中) | 在 [FeatureFlagUtil.m:85-169](longbridge-ios-app/LBPod/lbusercenterbizkit/LBUserBizKit/LBConfig/FeatureFlagUtil/FeatureFlagUtil.m#L85-L169) keys 数组中注册 |
| **品牌前置过滤** | 客户端额外加一道 `currentAccountChannel == kAccountChannel_LBSG` 判断作为**双保险**:即使后台误下发命中,LBSG 用户在客户端也强制走老客服 | 防御性兜底 |
| **一键回滚总闸** | **新增独立租户开关 key(命名 TBD)**,通过 `LBTenantConfigCenter functionSwitchForKey:` 读取。关掉 → 全部回退老客服,无视灰度命中状态 | 与灰度 key 解耦,可独立熔断;**key 命名待与后端对齐** |
| **未登录用户处理** | **暂定走老客服**(未登录状态下后台无法基于 user 维度判断灰度)。**TBD — 待与产品最终确认** | 推荐方案;最终行为以与产品对齐为准 |
| **自研 IM URL 下发** | **新增独立租户 key(命名 TBD)**,通过 `LBTenantConfigCenter tenantResourceModelForKey:` 读取;客户端拼接 query 参数后打开 WebView | **key 命名待与后端对齐** |
| **JSBridge 设计** | **完全不新增 Bridge**,全部从现有 17 个 API 类里选用;具体能力到 Bridge 的映射由 H5 同学按需选择 | 边界由 H5 同学评估,Native 不预设 |
| **协议二次确认** | 不在 Native 拦截,由 H5 通过 query 参数 `from=app` + 服务端协议状态判断,端内首次进入弹自家协议 | 协议本就是 H5 业务,Native 不参与状态判断 |
| **未读数通道** | 灰度命中:消息中心客服条目 + 铃铛 badge 数据源切换为「**新长连接 push + HTTP 兜底**」;未命中走 `QiyuHelper`(原行为) | **HTTP 接口、长连接 client、topic 命名 全部 TBD — 待与后端对齐** |
| **推送通道** | 服务端通过 APNs 直接下发,客户端仅负责「收到 → 路由 → badge 跟随系统 `aps.badge`」 | **payload 结构 TBD — 待与后端对齐**;`message_type`、`link` 字段格式待定 |
| **合并/频控/静默/脱敏** | **全部服务端承担,客户端不参与** | 已确认 |
| **多设备 badge 同步** | A 设备已读后调 server mark_as_read → 服务端推 unread=0 给所有端 | 客户端不计算 unread,只接收 |
| **附件能力** | 复用现有 `LBMediaJSBridgeApi`、`LBFileJSBridgeApi`、`LBPermissionJSBridgeApi`,**不开发新 API** | 现有通路已覆盖 |
| **新老切换数据迁移** | **不做客户端迁移**:历史会话留在七鱼侧供合规审计;新会话写新系统。客户端在命中灰度时不再展示老会话 | 合规要求历史保留;客户端无双写复杂度 |
| **回滚** | 关闭租户总闸开关 → 全部回退老客服;无需发版 | 一键回滚 |

---

## 四、功能点清单与实现方案

### F1 灰度命中判定 ✓

**新增文件:** `longbridge-ios-app/LBPod/LBExtraBizKit/LBExtraBizKit/Classes/CustomerService/LBCustomerServiceManager.{h,m}`

> Pod 归属已确认:**LBExtraBizKit**(与 QiyuHelper 同 Pod),便于后续七鱼下线后整体清理。

**核心方法:**
```objc
@interface LBCustomerServiceManager : NSObject
+ (instancetype)sharedInstance;

/// 是否走自研 IM(总闸 + 品牌双保险 + 灰度 三重判定,任何一项不满足即返回 NO)
+ (BOOL)useSelfDevelopedIM;

/// 自研 IM 入口 URL(已带 query 参数);若未配置则返回 nil
+ (nullable NSString *)selfDevelopedIMURL;
@end
```

**判定逻辑:**
```objc
+ (BOOL)useSelfDevelopedIM {
    // 1. 总闸(一键回滚开关)— TBD: 待与后端对齐 key 命名
    if (![LBTenantConfigCenter functionSwitchForKey:@"<TBD: 总闸 key>"]) return NO;
    // 2. 品牌双保险:LBSG 用户即使后台误下发也强制走老客服
    if ([UserDataCenter.currentAccountChannel isEqualToString:kAccountChannel_LBSG]) return NO;
    // 3. 灰度命中
    return [FeatureFlagUtil featureFlagForKey:@"customer_service_hk"];
}
```

**联动改造:**
- [FeatureFlagUtil.m:85-169](longbridge-ios-app/LBPod/lbusercenterbizkit/LBUserBizKit/LBConfig/FeatureFlagUtil/FeatureFlagUtil.m#L85-L169) keys 数组**新增** `@"customer_service_hk"`
- 不增加便利方法到 FeatureFlagUtil(语义属于客服模块,放 `LBCustomerServiceManager`)

> **未登录处理**:`useSelfDevelopedIM` 在未登录状态下当前实现会因为后台无法下发命中而返回 NO,自然走老客服 — 与「未登录暂定走老客服」一致。**待与产品最终确认是否需要为未登录用户提供另外的灰度策略**。

---

### F2 路由分发改造 ✓

**改造文件:** [QiyuHelper.m:80-95](longbridge-ios-app/LBPod/LBExtraBizKit/LBExtraBizKit/Classes/QiYuHelper/QiyuHelper.m#L80-L95)

```objc
[LBRouter registerURLPattern:kLBCustomerService toModelHandler:^(LBRouterModel *m) {
    // 【新增】最前置:自研 IM 命中
    if ([LBCustomerServiceManager useSelfDevelopedIM]) {
        NSString *url = [LBCustomerServiceManager selfDevelopedIMURL];
        if (url.length) {
            [LBRouter openURL:url];   // LBWebViewModule 自动接管
            return;
        }
    }
    // === 以下保持原逻辑(七鱼 H5 / 原生兜底) ===
    NSString *customerServiceUrl = [LBTenantConfigCenter tenantResourceModelForKey:@"customer_service_URL_v2"].lauString;
    if (customerServiceUrl.length > 0) { [LBRouter openURL:customerServiceUrl]; return; }
    customerServiceUrl = [LBTenantConfigCenter tenantResourceModelForKey:@"customer_service_URL"].lauString;
    if (customerServiceUrl.length > 0) { [LBRouter openURL:customerServiceUrl]; return; }
    [LBRouter openURL:kLBNativeCustomerService];
}];
```

`kLBNativeCustomerService` 不变(留作直接走原生七鱼的兜底入口)。

---

### F3 自研 IM URL 拼接 ✓

`LBCustomerServiceManager` 内:

```objc
+ (NSString *)selfDevelopedIMURL {
    NSString *base = [LBTenantConfigCenter tenantResourceModelForKey:@"<TBD: URL key>"].lauString;
    if (!base.length) return nil;
    NSDictionary *q = @{
        @"from": @"app",
        @"platform": @"ios",
        @"ver": [AppSupportManager appVersion] ?: @"",
        @"lang": [LBCustomerServiceManager currentLangCode],     // zh-CN/zh-HK/en
        @"theme": [SakuraThemeManager isDarkMode] ? @"dark" : @"light",
        @"account_channel": UserDataCenter.currentAccountChannel ?: @"",
    };
    return [LBURLAppendUtil appendQuery:q toURL:base];
}
```

> **TBD — 待与后端对齐:**
> - URL 租户 key 的最终命名
> - query 参数列表(`from`/`platform`/`ver`/`lang`/`theme`/`account_channel` 是 Native 侧的提议,以 H5 同学接收能力为准)

> **协议二次确认**:H5 收到 `from=app` 后,从服务端拉「App 端协议同意状态」,未同意则弹 App 版协议;**Native 不参与协议状态判断**。

---

### F4 登录态注入与失效处理 ✓

**登录态注入复用现有能力**:`LBWebViewConfigManager` 监听 `UserDataCenter` 已自动注入 token Cookie,无新工作。

**登录态失效**:H5 调用 `router.openURL(@"lb://page/login/guide")` 跳现有登录引导。

**清单核对:**
- [x] 用户已登录 → 直接进 IM 会话页(无登录中断)
- [x] 客服侧用户身份 = App 身份(同一份 token Cookie)
- [x] 登录态失效:H5 收到 401 → 通过 `LBRouterJSBridgeApi` 跳登录页

---

### F5 退出/切换账号关闭 IM ✓

**Cookie 清理**:**复用现有退出逻辑,不额外处理**。`LBWebViewConfigManager` 在退出时已统一处理同域 Cookie。

**会话页强制关闭**:PDF 第 4 页要求「退出登录的瞬间,正在进行的 IM 会话立即结束并关闭页面」——这个不是 WebView 默认行为,需要客户端主动做。

`LBCustomerServiceManager` 实现 `<LBUserDataCenterDelegate>`,在 `+load` 注册:

```objc
+ (void)userDataCenter:(id)c userLoginStateChanged:(BOOL)isLogin automatically:(BOOL)auto {
    if (isLogin) return;
    // 主动 pop / dismiss 当前所有自研 IM WebView(按 host 识别)
    [self closeAllSelfIMWebViews];
}
```

**实现思路**:
- `LBCustomerServiceManager` 维护一个对自研 IM `LBWebView` 的 weak 引用集合
- 打开自研 IM URL 时(F2 路由分支命中时)记录 weak 引用
- 登出回调中遍历集合,调用 `dismissViewControllerAnimated:` 或 `popViewControllerAnimated:`

**多设备**:每台设备独立维护会话生命周期,服务端 session token 仅对该设备失效,**不影响别的设备**(无客户端跨设备逻辑)。

---

### F6 未读数(铃铛 + 客服条目) ⚠️ 部分 TBD

**改造点 1:** [MessageCenterViewController.m:111-136, 359-365](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/Controller/MessageCenterViewController.m)

未读数订阅来源根据灰度命中切换:

```objc
- (void)bindUnreadSubject {
    if ([LBCustomerServiceManager useSelfDevelopedIM]) {
        @weakify(self);
        [[LBCustomerServiceManager.sharedInstance unreadCountSubject]
            subscribeNext:^(NSNumber *count) {
                @strongify(self);
                self.customerServiceGroupModel.unread_count = count;
                [self refreshList];
            }];
    } else {
        // 原 QiyuHelper.allUnreadCountSubject 订阅(保留)
    }
}
```

**改造点 2:** [NewMineBadgeManager.h](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/LBNavView/NewMineBadgeManager.h) 新增「自研 IM 未读」一类聚合源,数据由 `LBCustomerServiceManager.unreadCountSubject` 推送,逻辑等同于现有 Qiyu 这一路。

**未读数据三路汇聚(架构层面):**

1. **App 启动 / 进入消息中心** → HTTP 拉一次未读接口(服务端单一真相源)
2. **长连接** → 订阅 IM 未读 topic,收到推送即更新
3. **进入会话页 / 已读** → H5 调后端 mark_as_read → 服务端 push 给所有端 unread=0,客户端被动接收

**TBD — 待与后端对齐**:
- 未读数 HTTP 接口路径与字段
- 长连接 client(LBWSClient / LBQWSClient / LBQWSTCPClient 选哪个)
- 长连接 topic 命名

> **多设备同步**:服务端在 mark_as_read 后向所有在线端推 unread=0,客户端被动接受,**不本地计算**。

---

### F7 推送(APNs)落地 ⚠️ payload 结构 TBD

**改造点:** 大概率不需要改 [LBAlicloudPushInitializer.m](longbridge-ios-app/LBPod/LBAppEntry/LBAppEntry/Forwarder/LBAlicloudPushInitializer.m)。现有逻辑已通过 payload `link` 字段自动 `[LBRouter openURL:link]` 落地;命中灰度自然走自研 IM,未命中走老客服。

**客户端职责仅 3 条:**
1. 收到推送 → 走现有 `link` 自动路由(无需改造)
2. badge 角标更新走系统 `aps.badge` 字段,无需客户端计算
3. 用户点击推送进入 IM → H5 完成会话定位 + mark_as_read,Native badge 由服务端下推清零

**前台收到推送 / 抑制横幅**:
- 用户在自研 IM 页面前台时,需抑制 APNs 横幅,改由 H5 内消息流实时刷新(走长连接)
- **实现方式**:H5 通过现有 17 个 Bridge 中的某一个(由 H5 同学评估,例如 `LBNavigatorJSBridgeApi` 或 `LBDefaultJSBridgeApi`)告知 Native「当前 IM 在前台」;`LBCustomerServiceManager` 接收到状态后在 `userNotificationCenter:willPresentNotification:` 中识别 customer_service IM 类型推送并抑制
- **TBD — 待 H5 同学评估**:具体复用哪个 Bridge

**TBD — 待与后端对齐:**
- payload 中 `message_type` 字段值
- `link` 字段格式(是否带 session_id 等)
- 是否使用 `aps.badge` 标准字段

> **服务端职责确认:** 30 秒合并、单日 5 条上限、静默时段、敏感信息脱敏 全部服务端承担,客户端不实现、不参与判断。

---

### F8 JSBridge 复用 ✓

**本方案不新增 JSBridge API 类。** 所需能力到现有 17 个 Bridge 的映射由 H5 同学按需选择,以下为可用能力清单:

| 自研 IM 需要的能力 | 复用 Bridge(候选) |
|---|---|
| H5 打埋点 | LBEventTrackingJSBridgeApi |
| 上传图片(相册/相机) | LBMediaJSBridgeApi + LBPermissionJSBridgeApi |
| 选择并上传文件 | LBFileJSBridgeApi(若需要) |
| 关闭 WebView | LBRouterJSBridgeApi |
| 跳登录引导 | LBRouterJSBridgeApi(`router.openURL`) |
| H5 调后端接口 | LBNetworkJSBridgeApi(若需要原生网络层) |
| 通知 Native「IM 前台/后台」 | 由 H5 评估,从 `LBNavigatorJSBridgeApi`、`LBDefaultJSBridgeApi` 等中选用 |
| 已读上报 / badge 清零信号 | 推荐由 H5 直接走后端,服务端 push unread=0 给所有端 — Native 不需要单独 Bridge |

> **协作约定:** 如果 H5 同学评估发现现有 17 个 Bridge **确实**无法覆盖某个能力,再回到本方案讨论是否新增。本方案默认全部复用。

---

### F9 长连接订阅 ⚠️ TBD

`LBCustomerServiceManager` 在 `userDataCenter:userLoginStateChanged:automatically:` 登录回调中订阅 IM 长连接 topic,前后台切换复用现有长连接 `autoReconnectBlock`。

**TBD — 待与后端对齐:**
- 走 `LBWSClient` / `LBQWSClient` / `LBQWSTCPClient` 哪一个 client
- 订阅的 topic 列表与命名
- 收到 push 后的消息体格式

> 联调期对齐后,在本节补具体订阅代码与解析逻辑。

---

### F10 暗夜模式 / 多语言 ✓

- **暗夜**:URL query `theme=dark|light`,主题切换时通过 `kSakuraSwitchThemeNotification` 通知,`LBCustomerServiceManager` 监听后由 H5 自行换肤(具体通信通道由 H5 评估)
- **多语言**:URL query `lang`(`zh-CN/zh-HK/en`),`CFLanguageManager` 已有取值;`CFLanguageChangedNotification` 触发时由 H5 决定是否 reload

---

### F11 埋点 7 个事件 ⚠️ 字段命名 TBD

| 事件 | 触发位置 | 负责端 |
|---|---|---|
| `im_entry_click` | `kLBCustomerService` 路由命中前 | **Native**(`LBCustomerServiceManager`) |
| `im_page_enter` | WebView didFinish | Native |
| `im_message_send` | H5 发送消息 | H5(`LBEventTrackingJSBridgeApi`) |
| `im_first_reply` | H5 收到坐席首条回复 | H5 |
| `im_session_end` | H5 会话结束 | H5 |
| `im_csat_submit` | H5 满意度提交 | H5 |
| `im_transfer_human` | H5 转人工 | H5 |

**TBD — 待与产品/数据对齐:**
- `im_entry_click.entry_scene_id` 取值规则(每个客服入口对应的 ID 枚举)
- `gray_result` 字段名与取值(`new`/`old` vs `is_new_im=true/false` vs 其他)
- 其他字段命名细则

`im_entry_click` 由 `LBCustomerServiceManager` 在 `useSelfDevelopedIM` 调用方处统一打点(包到一个 `+ (void)trackEntryClickWithScene:`)。

---

### F12 多机型适配 / 横竖屏 / 后台切换

**复用现有 LBWebViewModule 通用回归**,无需新建。

测试矩阵清单:
- iOS 版本:**13.0+**(跟随工程现有最低版本设定),覆盖 iOS 13 / 14 / 15 / 16 / 17 / 18 主流子版本
- 主流机型:iPhone SE 3 / iPhone 12 / 14 / 15 / 16 Pro / iPad mini
- **横竖屏**:LBHK 横竖屏跟随用户设置,**测试矩阵需覆盖横竖屏切换**
- 网络切换:Wi-Fi ↔ 4G、断网重连(长连接重连验证)
- 后台切换:home 退后台 5min 回前台,长连接 + 离线消息拉取验证

---

## 五、关键文件改动清单

### 新增

| 文件 | 用途 |
|---|---|
| `LBPod/LBExtraBizKit/LBExtraBizKit/Classes/CustomerService/LBCustomerServiceManager.{h,m}` | 灰度判定、URL 拼接、未读订阅、登出关页、长连接订阅、埋点入口 |

> 注意:**不新增任何 JSBridge API 类**(原方案中提议的新增已撤销)。

### 修改

| 文件 | 改动 |
|---|---|
| [QiyuHelper.m:80-95](longbridge-ios-app/LBPod/LBExtraBizKit/LBExtraBizKit/Classes/QiYuHelper/QiyuHelper.m#L80-L95) | `kLBCustomerService` handler 最前面新增自研 IM 命中分支 |
| [FeatureFlagUtil.m:85-169](longbridge-ios-app/LBPod/lbusercenterbizkit/LBUserBizKit/LBConfig/FeatureFlagUtil/FeatureFlagUtil.m#L85-L169) | keys 数组新增 `@"customer_service_hk"` |
| [MessageCenterViewController.m:111-136, 359-365](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/Controller/MessageCenterViewController.m) | 客服条目未读订阅按灰度切换数据源 |
| [NewMineBadgeManager.{h,m}](longbridge-ios-app/LBPod/LBMessageCenterBizKit/LBMessageCenterBizKit/Classes/MessageCenter/LBNavView/NewMineBadgeManager.h) | 铃铛 badge 数据源新增「自研 IM 未读」一路 |
| [LBAlicloudPushInitializer.m](longbridge-ios-app/LBPod/LBAppEntry/LBAppEntry/Forwarder/LBAlicloudPushInitializer.m) | 大概率无需改动(payload `link` 已自动路由);如需 message_type 维度埋点/前台抑制,在 `userNotificationCenter:willPresentNotification:` 加分支 |

### 配置(非代码,需后端配合下发)

| 配置 | 说明 | 状态 |
|---|---|---|
| 灰度 key `customer_service_hk` | 1% → 5% → 20% → 50% → 100%;客户端在 `featureFlagRequest` 注册 | ✓ 已确认 |
| 一键回滚总闸开关 key | 租户级别 boolean | TBD — 待与后端对齐 key 命名 |
| 自研 IM URL key | 租户级别 string | TBD — 待与后端对齐 key 命名 |
| APNs 推送 payload | message_type、link 格式 | TBD — 待与后端对齐 |
| 未读数 HTTP 接口 | 路径与字段 | TBD — 待与后端对齐 |
| 长连接 topic | client 选择 + topic 命名 | TBD — 待与后端对齐 |

---

## 六、TBD 待对齐清单(联调期阻塞项)

> 实施前必须把以下项全部对齐,否则联调期会阻塞。

| 项 | 对齐对象 | 影响范围 |
|---|---|---|
| 一键回滚总闸开关 key 命名 | 后端 / 配置中心 | F1 灰度判定 |
| 自研 IM URL 租户 key 命名 | 后端 / 配置中心 | F3 URL 拼接 |
| URL query 参数列表 | H5 同学 | F3 URL 拼接 |
| 未读数 HTTP 接口路径与字段 | 后端 | F6 未读数 |
| 长连接 client 选择(LBWSClient / LBQWSClient / LBQWSTCPClient) | 后端 | F6 未读数 / F9 长连接 |
| 长连接 topic 命名与消息体格式 | 后端 | F6 未读数 / F9 长连接 |
| APNs payload 字段(message_type、link 格式) | 后端 / 推送平台 | F7 推送 |
| 「IM 前台/后台」状态告知 Native 的 Bridge 选用 | H5 同学 | F7 前台抑制横幅 |
| 未登录用户灰度策略最终方案 | 产品 | F1 判定逻辑 |
| `im_entry_click.entry_scene_id` 枚举 | 产品 / 数据 | F11 埋点 |
| `gray_result` 字段名与取值 | 数据 | F11 埋点 |
| 其他埋点字段命名细则 | 数据 | F11 埋点 |

---

## 七、实施顺序(建议分 3 个 PR)

> **前置条件:** 「TBD 待对齐清单」中标 ⚠️ 的关键项至少完成第一轮对齐(总闸 key、URL key、长连接 client / topic、APNs payload)。

**PR 1 — 基础打通**(联调期可上线,默认灰度 0%)
1. 新增 `LBCustomerServiceManager`(灰度判定 + URL 拼接,先空实现订阅与清理)
2. `FeatureFlagUtil` keys 注册 `customer_service_hk`
3. `QiyuHelper` 路由 handler 注入分支
4. 自测:打开内部白名单,从我的页/消息中心点击客服 → 跳到自研 IM H5,完成登录态注入、收发消息

**PR 2 — 未读 + 推送**
1. 长连接 topic 订阅 + `unreadCountSubject`
2. `MessageCenterViewController` 客服条目数据源切换
3. `NewMineBadgeManager` 接入新数据源
4. APNs 落地校验 + 前台抑制横幅(由 H5 同学告知前后台状态)

**PR 3 — 退出清理 + 埋点 + 多端测试**
1. `LBUserDataCenterDelegate` 登出回调中关闭自研 IM WebView
2. 7 个埋点事件(Native 2 + H5 5)
3. 暗夜/多语言切换通知透传
4. 多机型测试矩阵 + 灰度演练

---

## 八、回滚预案

| 触发条件 | 操作 | 生效时长 |
|---|---|---|
| 严重问题 | 后台关闭一键回滚总闸开关 | 用户下次启动或下次拉灰度即生效(< 5 分钟) |
| 局部问题 | 灰度比例调回低分位(50% → 5%) | 同上 |
| 紧急熔断 | 同时关闭灰度 key + 总闸开关 | < 5 分钟 |

回滚后所有入口自动回到老客服(七鱼 H5 / 原生 QYSDK),无需发版。

---

## 九、验证方案

### 9.1 端到端验收

| 验收点 | 操作 | 期望 |
|---|---|---|
| 灰度命中 | 后台把测试账号设为灰度命中 → App 杀进程冷启 → 任意客服入口点击 | 进入自研 IM H5,URL query 含 `from=app&ver=...` |
| 未命中 | 后台撤销灰度 → 杀进程 → 任意入口 | 走老客服(七鱼 H5 或原生) |
| LBSG 双保险 | 后台误下发命中给 LBSG 用户 | 客户端兜底,LBSG 仍走老客服 |
| 入口一致性 | 命中状态下,所有客服入口全部点过 | 全部进自研 IM |
| 登录态 | 已登录用户点入口 | 直接进会话页,无登录中断;客服侧拿到的 user_id = App user_id |
| 协议二次确认 | App 内首次进入 | H5 弹 App 端协议(由 H5 处理) |
| 退出账号 | IM 会话中点退出 | 会话页立即关闭;二次进入无残留未读 |
| 切账号 | 切换到账号 B | 新会话历史 = 账号 B,badge 不残留 |
| 未读数 | 服务端推 unread=3 | 铃铛 + 客服条目均显示 3,进入 IM 后立即清零 |
| 多设备 | A 设备已读 | B 设备 badge 在 < 5s 内清零 |
| 推送(后台) | 服务端推自研 IM 类型 | 收到横幅,点击进入对应会话 |
| 推送(前台 IM 内) | 服务端推 | 无横幅,IM 内消息列表实时更新 |
| 附件 | IM 内点上传图片 | 弹原生权限 → 选图 → 上传成功 |
| 暗夜 | 切系统暗夜 | H5 主题跟随切换 |
| 多语言 | 切 zh-HK / en | H5 文案跟随 |
| 横竖屏 | 切到横屏 | IM 页面正常布局 |
| 回滚 | 关一键总闸开关 | 5 分钟内全部入口回老客服 |

### 9.2 自动化/回归

- 现有 LBWebViewModule 单测/UI 测试不受影响
- 灰度判定逻辑加单测:命中 / 不命中 / LBSG 用户 / 总闸关 / 未登录 等组合

### 9.3 线上观测

- 埋点看板:`im_entry_click` 中灰度命中字段(命名 TBD)占比对照灰度比例
- 进入失败率:`im_page_enter` 与 `im_entry_click` 比例 < 100% 即 H5 加载异常
- 首响时长 P95(`im_first_reply` 中字段)
- 推送送达率:服务端发出量 vs APNs feedback

---

## 十、Roadmap 对齐

| 阶段 | 关键交付(本方案对应) |
|---|---|
| 准备期 | F1/F2/F3 完成、TBD 清单关键项首轮对齐、F11 埋点 SDK 确认、合规审核(协议) |
| 联调期 | F4-F10 全量联调、客服坐席工作台 E2E、TBD 清单全部清零 |
| 内部灰度 | 公司白名单(产品/研发/客服内测),全场景跑通 |
| 逐步灰度 | 1% → 5% → 20% → 50% → 100%,每档观察 ≥ 24h |

---

> **下一步:** 把第六章「TBD 待对齐清单」分发给后端、产品、数据、H5 同学,完成首轮对齐后回填本文档对应位置,再启动 PR 1 编码。
