import AnalyticsController from './AnalyticsController'
import BacktestController from './BacktestController'
import BillingController from './BillingController'
import DashboardController from './DashboardController'
import InternalNotificationController from './InternalNotificationController'
import Settings from './Settings'
import StrategyController from './StrategyController'
import WalletController from './WalletController'

const Controllers = {
    DashboardController: Object.assign(DashboardController, DashboardController),
    StrategyController: Object.assign(StrategyController, StrategyController),
    WalletController: Object.assign(WalletController, WalletController),
    BacktestController: Object.assign(BacktestController, BacktestController),
    BillingController: Object.assign(BillingController, BillingController),
    AnalyticsController: Object.assign(AnalyticsController, AnalyticsController),
    InternalNotificationController: Object.assign(InternalNotificationController, InternalNotificationController),
    Settings: Object.assign(Settings, Settings),
}

export default Controllers