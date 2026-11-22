import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import HeatsTableController from "./HeatsTable.controller";
import RacesTableController from "./RacesTable.controller";

/**
 * Race-entry detail view with keyboard and event-bus navigation through the selected races list.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RaceDetailsController extends BaseController {
  private static readonly RACE_ENTRIES_MODEL: string = "raceEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);
  private raceId?: number;

  /** Registers the entries model, visibility delegates, and route-triggered initial loading. */
  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), RaceDetailsController.RACE_ENTRIES_MODEL);

    super
      .getRouter()
      ?.getRoute(Routes.RaceDetails)
      ?.attachPatternMatched((event: Route$PatternMatchedEvent) => {
        super.runAsync(this.onPatternMatched(event), "Failed to load race details:");
      }, this);
  }

  /**
   * Captures the route's race ID before awaiting the initial load.
   * Loading starts here, not in {@link onBeforeShow}, so direct links have an ID before the request URL is built.
   * {@link updateNavOnPatternMatched} checks shared selection state before enabling list navigation.
   *
   * @param event - Route match containing `raceId` in its arguments.
   * @returns A promise resolving after the initial race-detail request settles.
   */
  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.raceId = (event.getParameter("arguments") as any)?.raceId;
    this.updateNavOnPatternMatched();
    await this.loadRaceModel();
  }

  /**
   * Hides the prev/next/first/last buttons when the user reaches the detail
   * view via a deep link instead of via the races list — see {@link onPatternMatched}.
   */
  private updateNavOnPatternMatched(): void {
    const race: any = super.getComponentJSONModel(RacesTableController.RACE_MODEL).getData();
    const cameFromList: boolean = race?.id != null && String(race.id) === String(this.raceId);
    if (!cameFromList) {
      const navData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: undefined };
      super.getComponentJSONModel(RacesTableController.RACE_NAV_MODEL).setData(navData);
    }
  }

  /** Enables list-change updates and keyboard navigation only while this detail view is visible. */
  private onBeforeShow(): void {
    super.getEventBus()?.subscribe("race", "itemChanged", this.onItemChanged, this);
    globalThis.addEventListener("keydown", this.keyListener);
  }

  /** Removes visibility-scoped listeners and clears the route ID before leaving the detail view. */
  private onBeforeHide(): void {
    globalThis.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("race", "itemChanged", this.onItemChanged, this);
    delete this.raceId;
  }

  /** Uses the stored navigation back target, falling back to the races list when none is set. */
  onNavBack(): void {
    const navData: NavigationData | undefined = super
      .getComponentJSONModel(RacesTableController.RACE_NAV_MODEL)
      .getData();
    if (navData?.back) {
      super.navBack(navData.back);
    } else {
      super.navBack(Routes.Races);
    }
  }

  onFirstPress(): void {
    super.getEventBus()?.publish("race", "first", {});
  }

  onPreviousPress(): void {
    super.getEventBus()?.publish("race", "previous", {});
  }

  onNextPress(): void {
    super.getEventBus()?.publish("race", "next", {});
  }

  onLastPress(): void {
    super.getEventBus()?.publish("race", "last", {});
  }

  /**
   * Reloads race entries and restores the refresh button even if loading rejects.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   */
  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadRaceModel()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh race details:"
    );
  }

  /**
   * Opens an entry's first heat, hiding heat-list navigation and retaining race details as the back target.
   *
   * @param event - Selection event whose list item is bound to an entry with its heat collection.
   */
  onEntriesItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(
        RaceDetailsController.RACE_ENTRIES_MODEL
      );
      const entry: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      if (entry?.heats?.length > 0) {
        const heat: any = entry.heats[0];

        const heatNavData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: Routes.RaceDetails };
        super.getComponentJSONModel(HeatsTableController.HEAT_NAV_MODEL).setData(heatNavData);

        super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).setData(heat);
        super.navToHeatDetails(heat.id);
      }
    }
  }

  /**
   * Loads race entries, preferring a selected race ID from the component model over the captured route ID.
   *
   * @returns Whether the race-detail request succeeded.
   */
  private async loadRaceModel(): Promise<boolean> {
    const race: any = super.getComponentJSONModel(RacesTableController.RACE_MODEL).getData();
    if (race?.id) {
      this.raceId = race.id;
    }
    const url: string = `/api/races/${this.raceId}`;
    const entriesModel = super.getViewJSONModel(RaceDetailsController.RACE_ENTRIES_MODEL);
    return await super.updateJSONModel(entriesModel, url);
  }

  /**
   * Reloads entries after the list controller changes the selected race.
   *
   * @param _channelId - Publishing channel; unused because this handler only listens to race changes.
   * @param _eventId - Event name; unused because this handler only listens to selection changes.
   * @param _parametersMap - Event payload; unused because the selected race is read from the component model.
   */
  private onItemChanged(_channelId: string, _eventId: string, _parametersMap: any): void {
    super.runAsync(this.loadRaceModel(), "Failed to load race details:");
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
