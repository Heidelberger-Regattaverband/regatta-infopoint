import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import HeatsTableController from "./HeatsTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatDetailsController extends BaseController {

  private static readonly ENTRIES_MODEL: string = "heatEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);
  private heatId?: number;

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), HeatDetailsController.ENTRIES_MODEL);

    super.getRouter()?.getRoute("heatDetails")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  /**
   * Captures the {@code heatId} from the route pattern *and* triggers the
   * initial data load. Doing the load here (rather than in {@link onBeforeShow})
   * guarantees `this.heatId` is set before `loadHeatModel` builds the URL —
   * otherwise on a deep-link navigation `onBeforeShow` fires *before* the
   * pattern handler, producing `/api/heats/undefined`.
   *
   * On a *deep-link* navigation (URL typed/bookmarked, no preceding heats list)
   * the prev/next/first/last buttons cannot work because there is no list to
   * traverse. Detect this by checking whether the component-level `heat` model
   * carries a matching id from a previous `HeatsTable` selection; if not, mark
   * the nav model as `disabled` so the view hides those buttons.
   */
  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.heatId = (event.getParameter("arguments") as any)?.heatId;
    this.updateNavOnPatternMatched();
    await this.loadHeatModel();
  }

  /**
   * Hides the prev/next/first/last buttons when the user reaches the detail
   * view via a deep link instead of via the heats list — see {@link onPatternMatched}.
   */
  private updateNavOnPatternMatched(): void {
    const heat: any = super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).getData();
    const cameFromList: boolean = heat?.id != null && String(heat.id) === String(this.heatId);
    if (!cameFromList) {
      const navData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: undefined };
      super.getComponentJSONModel(HeatsTableController.HEAT_NAV_MODEL).setData(navData);
    }
  }

  private onBeforeShow(): void {
    super.getEventBus()?.subscribe("heat", "itemChanged", this.onItemChanged, this);
    window.addEventListener("keydown", this.keyListener);
  }

  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("heat", "itemChanged", this.onItemChanged, this);
    delete this.heatId;
  }

  onNavBack(): void {
    const navData: NavigationData | undefined = super.getComponentJSONModel(HeatsTableController.HEAT_NAV_MODEL).getData();
    if (navData?.back) {
      super.navBack(navData.back);
    } else {
      super.navBack("heats");
    }
  }

  onFirstPress(): void {
    super.getEventBus()?.publish("heat", "first", {});
  }

  onPreviousPress(): void {
    super.getEventBus()?.publish("heat", "previous", {});
  }

  onNextPress(): void {
    super.getEventBus()?.publish("heat", "next", {});
  }

  onLastPress(): void {
    super.getEventBus()?.publish("heat", "last", {});
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadHeatModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  private async loadHeatModel(): Promise<boolean> {
    const heat: any = super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).getData();
    if (heat?.id) {
      this.heatId = heat.id;
    };
    const url: string = `/api/heats/${this.heatId}`;
    return await super.updateJSONModel(super.getViewJSONModel(HeatDetailsController.ENTRIES_MODEL), url);
  }

  private async onItemChanged(channelId: string, eventId: string, parametersMap: any): Promise<void> {
    await this.loadHeatModel();
  }

  private onKeyDown(event: KeyboardEvent): void {
    switch (event.key) {
      case "F5":
        event.preventDefault();
        break;
      case "ArrowLeft":
        this.onPreviousPress();
        break;
      case "ArrowRight":
        this.onNextPress();
        break;
      case "ArrowUp":
      case "Home":
        this.onFirstPress();
        break;
      case "ArrowDown":
      case "End":
        this.onLastPress();
        break;
    }
  }

}