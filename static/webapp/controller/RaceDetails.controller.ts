import Button, { Button$PressEvent } from "sap/m/Button";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import Context from "sap/ui/model/Context";
import RacesTableController from "./RacesTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RaceDetailsController extends BaseController {

  private static readonly ENTRIES_MODEL: string = "raceEntries";

  readonly formatter: Formatter = Formatter;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), RaceDetailsController.ENTRIES_MODEL);
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
  }

  onNavBack(): void {
    const data = (super.getComponentModel("race") as JSONModel).getData();
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

  onRegistrationsItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(RaceDetailsController.ENTRIES_MODEL);
      const registration: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      if (registration?.heats?.length > 0) {
        const heat: any = registration.heats[0];
        heat._nav = { disabled: true, back: "raceDetails" };

        (super.getComponentModel("heat") as JSONModel).setData(heat);
        super.navToHeatDetails(heat.id);
      }
    }
  }

  private async loadRaceModel(): Promise<boolean> {
    const race: any = (super.getComponentModel(RacesTableController.RACE_MODEL) as JSONModel).getData();
    const url: string = `/api/races/${race.id}`;
    return await super.updateJSONModel(super.getViewModel(RaceDetailsController.ENTRIES_MODEL) as JSONModel, url, super.getView());
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
