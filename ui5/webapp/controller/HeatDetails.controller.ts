import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import HeatsTableController from "./HeatsTable.controller";

/**
 * Heat-entry detail view with keyboard and event-bus navigation through the selected heats list.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatDetailsController extends BaseController {
  private static readonly ENTRIES_MODEL: string = "heatEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);
  private heatId?: number;

  /** Registers the entries model, visibility delegates, and route-triggered initial loading. */
  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), HeatDetailsController.ENTRIES_MODEL);

    super
      .getRouter()
      ?.getRoute(Routes.HeatDetails)
      ?.attachPatternMatched((event: Route$PatternMatchedEvent) => {
        super.runAsync(this.onPatternMatched(event), "Failed to load heat details:");
      }, this);
  }

  /**
   * Captures the route's heat ID before awaiting the initial load.
   * Loading starts here, not in {@link onBeforeShow}, so direct links have an ID before the request URL is built.
   * {@link updateNavOnPatternMatched} checks shared selection state before enabling list navigation.
   *
   * @param event - Route match containing `heatId` in its arguments.
   * @returns A promise resolving after the initial heat-detail request settles.
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

  /** Enables list-change updates and keyboard navigation only while this detail view is visible. */
  private onBeforeShow(): void {
    super.getEventBus()?.subscribe("heat", "itemChanged", this.onItemChanged, this);
    window.addEventListener("keydown", this.keyListener);
  }

  /** Removes visibility-scoped listeners and clears the route ID before leaving the detail view. */
  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("heat", "itemChanged", this.onItemChanged, this);
    delete this.heatId;
  }

  /** Uses the stored navigation back target, falling back to the heats list when none is set. */
  onNavBack(): void {
    const navData: NavigationData | undefined = super
      .getComponentJSONModel(HeatsTableController.HEAT_NAV_MODEL)
      .getData();
    if (navData?.back) {
      super.navBack(navData.back);
    } else {
      super.navBack(Routes.Heats);
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

  /**
   * Reloads heat entries and restores the refresh button on both success and failure.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   * @returns A promise resolving after loading, feedback, and button restoration, including handled failures.
   */
  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: boolean = await this.loadHeatModel();
      super.showDataUpdatedMessage(succeeded);
    } catch (error) {
      console.error("Failed to refresh heat details:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  /**
   * Loads heat entries, preferring a selected heat ID from the component model over the captured route ID.
   *
   * @returns Whether the heat-detail request succeeded.
   */
  private async loadHeatModel(): Promise<boolean> {
    const heat: any = super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).getData();
    if (heat?.id) {
      this.heatId = heat.id;
    }
    const url: string = `/api/heats/${this.heatId}`;
    return await super.updateJSONModel(super.getViewJSONModel(HeatDetailsController.ENTRIES_MODEL), url);
  }

  /**
   * Reloads entries after the list controller changes the selected heat.
   *
   * @param _channelId - Publishing channel; unused because this handler only listens to heat changes.
   * @param _eventId - Event name; unused because this handler only listens to selection changes.
   * @param _parametersMap - Event payload; unused because the selected heat is read from the component model.
   */
  private onItemChanged(_channelId: string, _eventId: string, _parametersMap: any): void {
    super.runAsync(this.loadHeatModel(), "Failed to load heat details:");
  }

  /**
   * Maps arrow/Home/End keys to list navigation and suppresses the browser's F5 refresh.
   *
   * @param event - Browser keyboard event whose key selects the navigation command.
   */
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
