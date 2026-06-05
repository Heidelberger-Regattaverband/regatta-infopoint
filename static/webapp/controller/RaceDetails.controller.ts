import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import HeatsTableController from "./HeatsTable.controller";
import RacesTableController from "./RacesTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RaceDetailsController extends BaseController {

  private static readonly RACE_ENTRIES_MODEL: string = "raceEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);
  private raceId?: number;

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), RaceDetailsController.RACE_ENTRIES_MODEL);

    super.getRouter()?.getRoute("raceDetails")?.attachPatternMatched(
      async (event: Route$PatternMatchedEvent) => await this.onPatternMatched(event), this);
  }

  /**
   * Captures the {@code raceId} from the route pattern *and* triggers the
   * initial data load. Doing the load here (rather than in {@link onBeforeShow})
   * guarantees `this.raceId` is set before `loadRaceModel` builds the URL —
   * otherwise on a deep-link navigation `onBeforeShow` fires *before* the
   * pattern handler, producing `/api/races/undefined`.
   *
   * On a *deep-link* navigation (URL typed/bookmarked, no preceding races list)
   * the prev/next/first/last buttons cannot work because there is no list to
   * traverse. Detect this by checking whether the component-level `race` model
   * carries a matching id from a previous `RacesTable` selection; if not, mark
   * the nav model as `disabled` so the view hides those buttons.
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

  private onBeforeShow(): void {
    super.getEventBus()?.subscribe("race", "itemChanged", this.onItemChanged, this);
    globalThis.addEventListener("keydown", this.keyListener);
  }

  private onBeforeHide(): void {
    globalThis.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("race", "itemChanged", this.onItemChanged, this);
    delete this.raceId;
  }

  onNavBack(): void {
    // Read the back-target from the dedicated nav model (cf. review issue #4),
    // not from the bound race data object.
    const navData: NavigationData | undefined = super.getComponentJSONModel(RacesTableController.RACE_NAV_MODEL).getData();
    if (navData?.back) {
      super.navBack(navData.back);
    } else {
      super.navBack("races");
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

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadRaceModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  onEntriesItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(RaceDetailsController.RACE_ENTRIES_MODEL);
      const entry: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      if (entry?.heats?.length > 0) {
        const heat: any = entry.heats[0];

        // Drive nav-button visibility/back-target through the dedicated heatNav
        // model so we never mutate the heat data object (cf. review issue #4).
        const heatNavData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: "raceDetails" };
        super.getComponentJSONModel(HeatsTableController.HEAT_NAV_MODEL).setData(heatNavData);

        super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).setData(heat);
        super.navToHeatDetails(heat.id);
      }
    }
  }

  private async loadRaceModel(): Promise<boolean> {
    const race: any = super.getComponentJSONModel(RacesTableController.RACE_MODEL).getData();
    if (race?.id) {
      this.raceId = race.id;
    };
    const url: string = `/api/races/${this.raceId}`;
    const entriesModel = super.getViewJSONModel(RaceDetailsController.RACE_ENTRIES_MODEL);
    return await super.updateJSONModel(entriesModel, url);
  }

  private async onItemChanged(channelId: string, eventId: string, parametersMap: any): Promise<void> {
    await this.loadRaceModel();
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
