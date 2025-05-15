import MyComponent from "de/regatta_hd/infoportal/Component";
import * as $ from "jquery";
import { LatLng } from "leaflet";
import MessageBox from "sap/m/MessageBox";
import MessageToast from "sap/m/MessageToast";
import Control from "sap/ui/core/Control";
import EventBus from "sap/ui/core/EventBus";
import Controller from "sap/ui/core/mvc/Controller";
import View from "sap/ui/core/mvc/View";
import History from "sap/ui/core/routing/History";
import Router from "sap/ui/core/routing/Router";
import JSONModel from "sap/ui/model/json/JSONModel";
import Model, { Model$RequestFailedEventParameters } from "sap/ui/model/Model";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class BaseController extends Controller {

  /**
   * Shows a message toast with the result of a data update operation.
   * @param succeeded data update succeeded or failed
   */
  showDataUpdatedMessage(succeeded: boolean): void {
    if (succeeded) { // NOSONAR
      MessageToast.show(this.i18n("msg.dataUpdated"));
      $(".sapMMessageToast").addClass("sapMMessageToastInfo");
    } else {
      MessageToast.show(this.i18n("msg.dataUpdateFailed"));
      $(".sapMMessageToast").addClass("sapMMessageToastDanger");
    }
  }

  /**
   * Convenience method for getting the secure context of the application.
   * @returns {boolean} whether the application is running in a secure context
   */
  isSecureContext(): boolean {
    console.debug(`isSecureContext: ${window.isSecureContext}`);
    return window.isSecureContext;
  }

  /**
   * Convenience method for accessing the content density class defined in the component.
   * @returns {string} the content density class
   */
  getContentDensityClass(): string {
    return this.getComponent()?.getContentDensityClass() ?? "sapUiSizeCozy";
  }

  /**
   * Convenience method for accessing the event bus for this component.
   * @returns {sap.ui.core.EventBus} the event bus for this component
   */
  getEventBus(): EventBus | undefined {
    return super.getOwnerComponent()?.getEventBus();
  }

  /**
   * Convenience method for accessing a component model.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  getComponentModel(name: string): Model | undefined {
    return super.getOwnerComponent()?.getModel(name);
  }

  /**
   * Convenience method for accessing the router.
   * @returns {sap.ui.core.routing.Router} the router for this component
   */
  getRouter(): Router {
    return this.getComponent().getRouter();
  }

  /**
   * Convenience method for getting the view model by name.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  getViewModel(name: string): Model | undefined {
    return super.getView()?.getModel(name);
  }

  /**
   * Convenience method for setting the view model.
   * @param {sap.ui.model.Model} model the model instance
   * @param {string} name the model name
   * @returns {sap.ui.mvc.View} the view instance
   */
  setViewModel(model: Model, name: string): View | undefined {
    return super.getView()?.setModel(model, name);
  }

  navBack(target: string): void {
    const previousHash: string | undefined = History.getInstance().getPreviousHash();
    if (previousHash) {
      window.history.back();
    } else {
      this.getRouter().navTo(target, {}, undefined, false /* history*/);
    }
  }

  /**
   * Translates the given key using the resource bundle of the component.
   * @param {string} key  The key to be translated
   * @param {any[]}  args The arguments to be passed to the translation
   * @returns {string} The translated text
   */
  i18n(key: string, args?: any[]): string {
    return this.getComponent()?.getResourceBundle().getText(key, args) ?? "";
  }

  async updateJSONModel(model: JSONModel, url: string, control?: Control): Promise<boolean> {
    control?.setBusy(true);
    try {
      await model.loadData(url);
      return true;
    } catch (error: any) {
      const params: Model$RequestFailedEventParameters = error as Model$RequestFailedEventParameters;
      MessageBox.error((params.statusCode ?? "") + ": " + params.statusText);
      return false;
    } finally {
      control?.setBusy(false);
    }
  }

  navToStartPage(): void {
    this.getRouter().navTo("startpage");
  }

  navToRaces(): void {
    this.getRouter().navTo("races");
  }

  navToRaceDetails(raceId: number): void {
    this.getRouter().navTo("raceDetails", { raceId: raceId });
  }

  navToHeats(): void {
    this.getRouter().navTo("heats");
  }

  navToHeatDetails(heatId: number): void {
    this.getRouter().navTo("heatDetails", { heatId: heatId });
  }

  navToClubs(): void {
    this.getRouter().navTo("clubs");
  }

  navToClubDetails(clubId: number): void {
    this.getRouter().navTo("clubDetails", { clubId: clubId }, false /* history*/);
  }

  navToAthletes(): void {
    this.getRouter().navTo("athletes");
  }

  navToAthleteDetails(athleteId: number): void {
    this.getRouter().navTo("athleteDetails", { athleteId: athleteId }, false /* history*/);
  }

  navToMap(location?: LatLng): void {
    let params: any = {};
    if (location) {
      params = { "lat": location.lat, "lng": location.lng };
    }
    this.getRouter().navTo("map", params);
  }

  async getFilters(): Promise<any> {
    return (await this.getComponent().getFilters()).getData();
  }

  async getActiveRegatta(): Promise<any> {
    return (await this.getComponent().getActiveRegatta()).getData();
  }

  private getComponent(): MyComponent {
    return super.getOwnerComponent() as MyComponent;
  }
}
