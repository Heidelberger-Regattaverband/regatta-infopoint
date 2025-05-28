import Button, { Button$PressEvent } from "sap/m/Button";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Context from "sap/ui/model/Context";
import RacesTableController from "./RacesTable.controller";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import HeatsTableController from "./HeatsTable.controller";

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
      (event: Route$PatternMatchedEvent) => this.onPatternMatched(event), this);
  }

  private onPatternMatched(event: Route$PatternMatchedEvent): void {
    this.raceId = (event.getParameter("arguments") as any)?.raceId;
  }

  private onBeforeShow(): void {
    this.loadRaceModel().then(() => {
      super.getEventBus()?.subscribe("race", "itemChanged", this.onItemChanged, this);
      window.addEventListener("keydown", this.keyListener);
    })
  }

  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
    super.getEventBus()?.unsubscribe("race", "itemChanged", this.onItemChanged, this);
    delete this.raceId;
  }

  onNavBack(): void {
    const data = (super.getComponentModel(RacesTableController.RACE_MODEL) as JSONModel).getData();
    if (data._nav?.back) {
      super.navBack(data._nav.back);
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
        heat._nav = { disabled: true, back: "raceDetails" };

        (super.getComponentModel(HeatsTableController.HEAT_MODEL) as JSONModel).setData(heat);
        super.navToHeatDetails(heat.id);
      }
    }
  }

  private async loadRaceModel(): Promise<boolean> {
    const race: any = (super.getComponentModel(RacesTableController.RACE_MODEL) as JSONModel).getData();
    if (race?.id) {
      this.raceId = race.id;
    };
    const url: string = `/api/races/${this.raceId}`;
    const entriesModel = super.getViewModel(RaceDetailsController.RACE_ENTRIES_MODEL) as JSONModel;
    return await super.updateJSONModel(entriesModel, url, super.getView());
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
