import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import Button, { Button$PressEvent } from "sap/m/Button";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RaceRegistrationsTable extends BaseController {

  formatter: Formatter = Formatter;
  private keyListener: (event: KeyboardEvent) => void;
  private readonly raceModel: JSONModel = new JSONModel();

  onInit(): void {
    // first initialize the view
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeShow: this.onBeforeShow, onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(this.raceModel, "raceRegistrations");

    super.getEventBus()?.subscribe("race", "itemChanged", this.onItemChanged, this);

    // bind keyListener method to this context to have access to navigation methods
    this.keyListener = this.onKeyDown.bind(this);
  }

  private onBeforeShow(): void {
    this.loadRaceModel().then(() => {
      window.addEventListener("keydown", this.keyListener);
    })
  }

  private onBeforeHide(): void {
    window.removeEventListener("keydown", this.keyListener);
  }

  onNavBack(): void {
    super.displayTarget("races");
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

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    const updated: boolean = await this.loadRaceModel();
    if (updated) {
      MessageToast.show(this.i18n("msg.dataUpdated"));
    }
    source.setEnabled(true);
  }

  private async loadRaceModel(): Promise<boolean> {
    const race: any = (super.getComponentModel("race") as JSONModel).getData();
    return await super.updateJSONModel(this.raceModel, `/api/races/${race.id}`, super.getView());
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
